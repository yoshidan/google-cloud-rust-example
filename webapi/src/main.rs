use crate::di::InjectedApi;
use actix_web::{middleware, App, HttpServer};
use anyhow::Context;
use google_cloud_default::WithAuthExt;
use google_cloud_example_lib::trace::Tracer;
use google_cloud_gax::conn::Environment;
use google_cloud_spanner::client::ClientConfig;
use std::env::set_var;
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio_util::sync::CancellationToken;

mod api;
mod application;
mod di;
mod domain;
mod infrastructure;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // ---- dummy setting
    set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    set_var("RUST_LOG", "info");
    let database = "projects/local-project/instances/test-instance/databases/local-database";
    // ---- dummy setting

    let config = ClientConfig::default().with_auth().await?;
    let mut tracer = Tracer::new(None).await;
    tracing::info!("Initializing server");
    let cancel = CancellationToken::new();
    let spanner_client = google_cloud_spanner::client::Client::new(database, config).await?;

    let dicon = actix_web::web::Data::new(InjectedApi::new(spanner_client.clone()));
    let _web_task = tokio::spawn(async move {
        let server = HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::default())
                .app_data(dicon.clone())
                .service(api::create_new_user)
                .service(api::get_user_inventory)
        })
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
