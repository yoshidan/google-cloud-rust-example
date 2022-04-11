use std::sync::Arc;
use google_cloud_auth::create_token_source;
use google_cloud_auth::token_source::TokenSource;
use opentelemetry::sdk::Resource;
use opentelemetry::runtime::Tokio;
use opentelemetry::sdk::trace::{Config, Sampler, Tracer, TracerProvider};
use tracing_subscriber::{EnvFilter, fmt, prelude::*, Registry};
use opentelemetry_stackdriver::{LogContext, StackDriverExporter, MonitoredResource, Authorizer, Error};
use tokio::task::JoinHandle;
use tonic::metadata::MetadataValue;
use tonic::Request;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_stackdriver::Stackdriver;
use tracing_subscriber::layer::Layered;
use warp::trace::Trace;
use async_trait::async_trait;

struct TraceAuthorizer {
    project_id: String,
    ts: Arc<dyn TokenSource>
}

impl TraceAuthorizer {
    async fn new(project_id: String) -> Self {
        let ts = create_token_source(google_cloud_auth::Config {
            audience: None,
            scopes:Some(&[
                "https://www.googleapis.com/auth/trace.append",
                "https://www.googleapis.com/auth/logging.write"
            ])
        }).await.unwrap();
        TraceAuthorizer {
            project_id,
            ts: Arc::from(ts)
        }
    }
}

#[async_trait]
impl Authorizer for TraceAuthorizer {
    type Error = opentelemetry_stackdriver::Error;

    fn project_id(&self) -> &str {
        self.project_id.as_str()
    }

    async fn authorize<T: Send + Sync>(&self, req: &mut Request<T>, scopes: &[&str]) -> Result<(), Self::Error> {
        let token = self.ts.token().await.map_err(|e| Error::Authorizer(e.into()))?.access_token;
        req.metadata_mut().insert(
            "authorization",
            MetadataValue::from_str(&format!("Bearer {}", token.as_str())).unwrap(),
        );
        Ok(())
    }
}

pub async fn create_tracer_provider(project_id: &str) -> (JoinHandle<()>, TracerProvider) {
    let mut builder = TracerProvider::builder();
    let mut j : JoinHandle<()>;
    if project_id != "local-project" {
        let log_context = LogContext {
            log_id: "cloud-trace-test".into(),
            resource: MonitoredResource::Global {
                project_id: project_id.to_string(),
            },
        };
        let auth = TraceAuthorizer::new(project_id.to_string()).await;
        let (exporter, driver) = StackDriverExporter::builder()
            .log_context(log_context)
            .build(auth)
            .await.unwrap();

        builder = builder.with_batch_exporter(exporter, Tokio);
        j = tokio::spawn(driver);
    }else {
        j = tokio::spawn(async {});
    }
    let provider = builder
        .with_config(Config {
            sampler: Box::new(Sampler::TraceIdRatioBased(1.0)),
            ..Default::default()
        })
        .build();

    return (j,provider);
}