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
use google_cloud_spanner::client::TxError::{GRPC, SessionError};
use prost_types::{Value, Timestamp};
use google_cloud_spanner::value::CommitTimestamp;
use chrono::{Utc, TimeZone, NaiveDateTime, DateTime};
use uuid::Uuid;
use std::convert::Infallible;
use google_cloud_spanner::key::Key;

mod model;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error(transparent)]
    GRPC(#[from] Status),
    #[error(transparent)]
    ParseError(#[from] RowError)
}

fn with_db(client: Arc<Client>) -> impl Filter<Extract = (Arc<Client>,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {

    let database = std::env::var("SPANNER_DSN").unwrap();

    env_logger::init();
    log::info!("start server");

    let client = Arc::new(Client::new(database).await.unwrap());
    let reader = warp::path!("read" / String)
        .and(with_db(client.clone()))
        .and_then(move |user_id, cl| read_handler(user_id, cl));
    let writer = warp::path!("write")
        .and(with_db(client.clone()))
        .and_then(move |cl| write_handler(cl));

    let routes = warp::get().and(reader.or(writer));

    let (tx, rx) = tokio::sync::oneshot::channel();
    let (_, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(([127, 0, 0, 1], 3031), async {
            log::info!("listening on http://127.0.0.1:3031");
            rx.await.ok();
            log::info!("shutdown server");
        });;

    tokio::spawn(server);

    tokio::signal::ctrl_c().await;
    let _ = tx.send(());
    client.close().await;
    log::info!("All the spanner sessions are deleted.");
}

async fn read_handler(user_id: String, client: Arc<Client>) -> Result<impl Reply, Rejection> {
    let tx = match client.read_only_transaction().await {
        Ok(tx) => tx,
        Err(e) => {
            let error_message =  "aaa".to_string();
            return Ok(warp::reply::with_status(
                warp::reply::html(error_message),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    match read(user_id, tx).await {
        Ok(rows)  => {
            Ok(warp::reply::with_status(
                warp::reply::html(format!("session size={}",client.session_count())),
                warp::http::StatusCode::OK,
            ))
        },
        Err(e) => {
            Ok(warp::reply::with_status(
                warp::reply::html(match e {
                    Error::GRPC(e)  => e.to_string(),
                    Error::ParseError(e) => {
                        log::error!("{:?}", e);
                        "parse error".to_string()
                    }

                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn read(user_id:String, mut tx: ReadOnlyTransaction) -> Result<Vec<String>, Error> {
    let mut stmt = Statement::new("SELECT * , \
            ARRAY (SELECT AS STRUCT * FROM UserItem WHERE UserId = @Param1 ) AS UserItem, \
            ARRAY (SELECT AS STRUCT * FROM UserCharacter WHERE UserId = @Param1 ) AS UserCharacter  \
            FROM User \
            WHERE UserId = @Param1");
    stmt.add_param("Param1", user_id);
    let mut reader = tx.query(stmt).await.unwrap();

    loop {
        let mut data = vec![];
        let row = match reader.next().await.map_err(|e| Error::GRPC(e))?{
            Some(row) => row,
            None => return Ok(data)
        };
        let user_id= row.column_by_name::<String>("UserId")
            .map_err(|e| Error::ParseError(e))?;
        let user_items= row.column_by_name::<Vec<model::UserItem>>("UserItem")
            .map_err(|e| Error::ParseError(e))?;
        let user_characters = row.column_by_name::<Vec<model::UserCharacter>>("UserCharacter")
            .map_err(|e| Error::ParseError(e))?;
        data.push(user_id);
    };
}


async fn write_handler(client: Arc<Client>) -> Result<impl Reply, Rejection> {
    let result = client.read_write_transaction_sync(|mut tx| {
        let new_user = model::User {
            user_id: Uuid::new_v4().to_string(),
            premium: true,
            updated_at: Utc::now(),
        };
        let new_user2 = model::User {
            user_id: Uuid::new_v4().to_string(),
            premium: false,
            updated_at: Utc::now(),
        };
        let m1 = insert_or_update_struct("User", new_user);
        let m2 = insert_or_update_struct("User", new_user2);
        tx.buffer_write(vec![m1,m2]);
        Ok(()) as Result<(), TxError>
    }).await;
    match result {
        Ok(result )  => {
            Ok(warp::reply::with_status(
                warp::reply::html(format!("result={}",result.0.unwrap().seconds)),
                warp::http::StatusCode::OK,
            ))
        },
        Err(e) => {
            Ok(warp::reply::with_status(
                warp::reply::html(match e {
                    TxError::GRPC(e)  => e.to_string(),
                    TxError::SessionError(e) => {
                        log::error!("{:?}", e);
                        "session error".to_string()
                    }

                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
