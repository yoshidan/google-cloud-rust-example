use actix_web::{middleware, App, HttpServer};
use anyhow::Context;
use chrono::Timelike;
use google_cloud_auth::Project;
use google_cloud_gax::cancel::CancellationToken;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::client::ClientConfig;
use google_cloud_pubsub::publisher::{Publisher, PublisherConfig};
use google_cloud_pubsub::subscriber::ReceivedMessage;
use google_cloud_spanner::client::RunInTxError;
use google_cloud_spanner::key::Key;
use google_cloud_spanner::reader::AsyncIterator;
use std::time::Duration;
use std::{env, time};
use std::env::set_var;

use google_cloud_spanner::value::Timestamp;
use google_cloud_storage::sign::SignedURLOptions;

use tokio::select;

use tokio::signal::unix::{signal, SignalKind};

use google_cloud_example_lib::trace::Tracer;
use crate::di::InjectedApi;

mod di;

#[derive(Debug)]
struct Config {
    spanner_dsn: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            spanner_dsn: "projects/local-project/instances/test-instance/databases/local-database".to_string(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // ---- dummy setting
    set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    set_var("RUST_LOG", "info");
    // ---- dummy setting

    let config = Config::default();

    let mut tracer = Tracer::default().await;
    tracing::info!("Initializing server");
    let cancel = CancellationToken::new();
    let spanner_client = google_cloud_spanner::client::Client::new(&config.spanner_dsn).await?;

    let dicon = actix_web::web::Data::new(InjectedApi::new(spanner_client.clone()));
    let web_task = tokio::spawn(async move {
        let server = HttpServer::new(move || App::new().wrap(middleware::Logger::default()).app_data(dicon.clone()))
            .bind(("0.0.0.0", 8100))?
            .run();
        tracing::info!("starting HTTP server at 0.0.0.0:8100");

        // Automatically shutdown gracefully.
        server.await.context("server error")
    });

    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    select! {
        _ = sigint.recv() => tracing::info!("SIGINT"),
        _ = sigterm.recv() => tracing::info!("SIGTERM"),
        _ = cancel.cancelled() => {},
    }
    if !cancel.is_cancelled() {
        cancel.cancel();
    }
    let _ = spanner_client.close();
    tracing::info!("Shutdown complete.");
    tracer.done().await;
    Ok(())
}
