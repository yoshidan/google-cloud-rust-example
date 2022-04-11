use google_cloud_spanner::client::Client;

use std::collections::HashMap;
use std::convert::Infallible;
use opentelemetry::trace::TracerProvider;

use tokio::signal::unix::{signal, SignalKind};
use tracing::instrument::WithSubscriber;
use tracing::log::Level;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use warp::Filter;

mod handler;
mod model;
mod init;

fn with_client(client: Client) -> impl Filter<Extract = (Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let database = std::env::var("SPANNER_DSN").unwrap();
    let project_id = std::env::var("PROJECT_ID").unwrap();

    let (j,provider) = init::create_tracer_provider(project_id.as_str()).await;
    tracing_subscriber::registry()
        .with( tracing_opentelemetry::layer().with_tracer(provider.tracer("tracing") ))
        .with(tracing_stackdriver::Stackdriver::new())
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    tracing::info!("Start server.");

    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();

    let client = Client::new(database).await.unwrap();

    //define routes
    let read_inventory_handler = warp::path!("ReadOnly")
        .and(warp::body::form())
        .and(with_client(client.clone()))
        .and_then(move |param: HashMap<String, String>, cl| {
            handler::read_inventory_handler(cl, param.get("user_id").unwrap().to_string())
        });
    let create_user_handler = warp::path!("CreateUser")
        .and(with_client(client.clone()))
        .and_then(move |cl| handler::create_user_handler(cl));
    let update_inventory_handler = warp::path!("ReadWrite")
        .and(warp::body::form())
        .and(with_client(client.clone()))
        .and_then(move |param: HashMap<String, String>, cl| {
            handler::update_inventory_handler(cl, param.get("user_id").unwrap().to_string())
        });
    let routes = warp::post().and(
        read_inventory_handler
            .or(create_user_handler)
            .or(update_inventory_handler),
    );

    // launch server
    let (tx, rx) = tokio::sync::oneshot::channel();
    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], 3031), async {
        tracing::info!("Listening on http://0.0.0.0:3031");
        rx.await.ok();
        tracing::info!("Shutdown server");
    });

    tokio::spawn(server);

    // wait for signal
    tokio::select! {
        _ = sigint.recv() => println!("SIGINT"),
        _ = sigterm.recv() => println!("SIGTERM"),
    };
    let _ = tx.send(());
    client.close().await;
    tracing::info!("All the spanner sessions are deleted.");
}
