use warp::{Filter, Reply, Rejection};
use std::collections::BTreeMap;
use std::sync::Arc;
use google_cloud_spanner::client::{Client, ClientConfig, TxError};
use google_cloud_spanner::mutation::{insert_or_update, insert_or_update_struct};
use google_cloud_spanner::statement::{Statement, ToStruct, Types, Kinds, ToKind};
use google_cloud_spanner::reader::{StatementReader, AsyncIterator, RowIterator};
use google_cloud_spanner::transaction::{ReadOptions};
use google_cloud_spanner::row::{Error as RowError, TryFromStruct, Struct};
use google_cloud_googleapis::Status;
use google_cloud_spanner::transaction_ro::ReadOnlyTransaction;
use google_cloud_spanner::client::TxError::{GRPC, InvalidSession};
use prost_types::{Value, Timestamp};
use google_cloud_spanner::value::CommitTimestamp;
use chrono::{Utc, TimeZone, NaiveDateTime, DateTime};
use uuid::Uuid;
use std::convert::Infallible;
use google_cloud_spanner::key::Key;
use tokio::signal::unix::{signal, SignalKind};

mod model;
mod handler;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error(transparent)]
    GRPC(#[from] Status),
    #[error(transparent)]
    ParseError(#[from] RowError)
}

fn with_client(client: Client) -> impl Filter<Extract = (Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {

    let database = std::env::var("SPANNER_DSN").unwrap();

    env_logger::init();
    log::info!("Start server.");

    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm= signal(SignalKind::terminate()).unwrap();

    let client = Client::new(database).await.unwrap();

    //define routes
    let read_inventory_handler= warp::path!("ReadInventory" / String)
        .and( with_client(client.clone()))
        .and_then(move |user_id, cl| handler::read_inventory_handler(cl, user_id));
    let create_user_handler = warp::path!("CreateUser")
        .and( with_client(client.clone()))
        .and_then(move |cl| handler::create_user_handler(cl));
    let update_inventory_handler = warp::path!("UpdateInventory" / String)
        .and( with_client(client.clone()))
        .and_then(move |user_id, cl| handler::update_inventory_handler(cl, user_id));
    let routes = warp::get().and(read_inventory_handler.or(create_user_handler).or(update_inventory_handler));

    // launch server
    let (tx, rx) = tokio::sync::oneshot::channel();
    let (_, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(([127, 0, 0, 1], 3031), async {
            log::info!("Listening on http://127.0.0.1:3031");
            rx.await.ok();
            log::info!("Shutdown server");
        });;

    tokio::spawn(server);

    // wait for signal
    tokio::select! {
        _ = sigint.recv() => println!("SIGINT"),
        _ = sigterm.recv() => println!("SIGTERM"),
    };
    let _ = tx.send(());
    client.close().await;
    log::info!("All the spanner sessions are deleted.");
}
