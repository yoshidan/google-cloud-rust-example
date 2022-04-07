use opentelemetry::sdk::Resource;
use opentelemetry::runtime::Tokio;
use opentelemetry::sdk::trace::{Config, Sampler,TracerProvider};
use opentelemetry::trace::TracerProvider as _;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use opentelemetry_stackdriver::{GcpAuthorizer, LogContext, StackDriverExporter, MonitoredResource};
use tracing_stackdriver::Stackdriver;

pub async fn init_trace(project_id: &str) {

    let log_context = LogContext {
        log_id: "cloud-trace-test".into(),
        resource: MonitoredResource::Global {
            project_id: project_id.to_string(),
        },
    };
    let authorizer = GcpAuthorizer::new().await.unwrap();
    let (exporter, driver) = StackDriverExporter::builder()
        .log_context(log_context)
        .build(authorizer)
        .await.unwrap();

    tokio::spawn(driver);

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter.clone(), Tokio)
        .with_config(Config {
            sampler: Box::new(Sampler::TraceIdRatioBased(1.0)),
            ..Default::default()
        })
        .build();

    let telemetry = tracing_opentelemetry::layer().with_tracer(provider.tracer("tracing"));
    tracing_subscriber::registry()
        .with(telemetry)
        .with( Stackdriver::default()) //GCP stackdriver format
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();
}