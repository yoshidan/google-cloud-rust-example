use std::sync::Arc;

use async_trait::async_trait;
use google_cloud_auth::token::DefaultTokenSourceProvider;
use google_cloud_token::{TokenSource, TokenSourceProvider};
use opentelemetry::runtime::Tokio;
use opentelemetry::sdk::trace::{Builder, Config, Sampler, TracerProvider};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_stackdriver::{Authorizer, Error, LogContext, MonitoredResource, StackDriverExporter};
use tokio::task::JoinHandle;
use tonic::metadata::MetadataValue;
use tonic::Request;
use tracing_stackdriver::CloudTraceConfiguration;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

struct TraceAuthorizer {
    project_id: String,
    ts: Arc<dyn TokenSource>,
}

impl TraceAuthorizer {
    async fn new(project_id: String) -> Self {
        let ts = DefaultTokenSourceProvider::new(google_cloud_auth::project::Config {
            audience: None,
            scopes: Some(&[
                "https://www.googleapis.com/auth/trace.append",
                "https://www.googleapis.com/auth/logging.write",
            ]),
        })
        .await
        .unwrap();
        TraceAuthorizer {
            project_id,
            ts: ts.token_source(),
        }
    }
}

#[async_trait]
impl Authorizer for TraceAuthorizer {
    type Error = opentelemetry_stackdriver::Error;

    fn project_id(&self) -> &str {
        self.project_id.as_str()
    }

    async fn authorize<T: Send + Sync>(&self, req: &mut Request<T>, _scopes: &[&str]) -> Result<(), Self::Error> {
        let token = self.ts.token().await.map_err(|e| Error::Authorizer(e.into()))?;
        req.metadata_mut()
            .insert("authorization", MetadataValue::from_str(token.as_str()).unwrap());
        Ok(())
    }
}

fn create_provider(builder: Builder) -> TracerProvider {
    builder
        .with_config(Config {
            sampler: Box::new(Sampler::AlwaysOn),
            ..Default::default()
        })
        .build()
}

pub async fn spawn(project_id: String) -> JoinHandle<()> {
    let log_context = LogContext {
        log_id: "cloud-trace-example".into(),
        resource: MonitoredResource::Global {
            project_id: project_id.to_string(),
        },
    };
    let auth = TraceAuthorizer::new(project_id.to_string()).await;
    let (exporter, driver) = StackDriverExporter::builder()
        .log_context(log_context)
        .num_concurrent_requests(2)
        .build(auth)
        .await
        .unwrap();

    let provider = create_provider(TracerProvider::builder().with_batch_exporter(exporter, Tokio));
    tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("tracing")))
        .with(tracing_stackdriver::layer().enable_cloud_trace(CloudTraceConfiguration { project_id }))
        .with(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    return tokio::spawn(driver);
}

pub fn start() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
}
