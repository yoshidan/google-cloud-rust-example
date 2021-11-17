use warp::{Filter, Reply, Rejection};
use std::collections::BTreeMap;
use std::sync::Arc;
use google_cloud_spanner::client::{Client, ClientConfig, TxError};
use google_cloud_spanner::mutation::{insert_struct, update};
use google_cloud_spanner::statement::{Statement, ToStruct, Types, Kinds, ToKind};
use google_cloud_spanner::reader::{StatementReader, AsyncIterator, RowIterator};
use google_cloud_spanner::transaction::{ReadOptions, Transaction};
use google_cloud_spanner::row::{Error as RowError, TryFromStruct, Struct};
use google_cloud_googleapis::Status;
use google_cloud_spanner::transaction_ro::ReadOnlyTransaction;
use google_cloud_spanner::client::TxError::{GRPC, SessionError};
use prost_types::{Value, Timestamp};
use google_cloud_spanner::value::CommitTimestamp;
use chrono::{Utc, TimeZone, NaiveDateTime, DateTime};
use uuid::Uuid;
use std::convert::Infallible;
use google_cloud_spanner::key::{Key, KeySet};
use crate::model;
use std::ops::DerefMut;
use google_cloud_gax::invoke::AsGrpcStatus;
use crate::model::UserItem;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error(transparent)]
    ParseError(#[from] RowError),
    #[error("no data found user_id = {0}")]
    NoDataFound(String ),
    #[error(transparent)]
    TxError(#[from] TxError),
}

impl AsGrpcStatus for Error {
    fn as_status(&self) -> Option<&Status> {
        match self {
            Error::TxError(TxError::GRPC(s)) => Some(s),
            _ => None
        }
    }
}

impl From<Status> for Error {
    fn from(s: Status) -> Self {
        return Error::TxError(TxError::GRPC(s));
    }
}

pub async fn create_user_handler(client: Arc<Client>) -> Result<impl Reply, Rejection> {
    let mut ms = vec![];
    let user_id =  Uuid::new_v4().to_string();
    let user = model::User {
        user_id: user_id.to_string(),
        premium: true,
        updated_at: Utc::now(),
    };
    ms.push(insert_struct("User", user));
    for i in 0..10 {
        let user_item = model::UserItem {
            user_id: user_id.to_string(),
            item_id: i,
            quantity: 0,
            updated_at: Utc::now()
        };
        ms.push(insert_struct("UserItem", user_item));

        let user_character = model::UserCharacter{
            user_id: user_id.to_string(),
            character_id: i,
            level: 1,
            acquired_at: Utc::now(),
            updated_at: Utc::now()
        };
        ms.push(insert_struct("UserCharacter", user_character));
    }

    match client.apply(ms).await {
        Ok(_)  => {
            Ok(warp::reply::with_status(
                warp::reply::html(user_id.to_string()),
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

pub async fn update_inventory_handler(client: Arc<Client>, user_id: String) -> Result<impl Reply, Rejection> {
    let tx_result = client.read_write_transaction(|mut tx| async {
        let result : Result<(), Error> = async {
            let mut reader = tx.read("UserItem", vec!["UserId", "ItemId", "Quantity"], Key::one(user_id.to_string())).await?;
            let ms  = loop {
                let mut ms = vec![];
                let row = reader.next().await?;
                match row {
                    Some(row) => {
                        let item_id = row.column_by_name::<i64>("ItemId")?;
                        let quantity = row.column_by_name::<i64>("Quantity")?;
                        ms.push(update("UserItem", vec!["Quantity"], vec![
                            user_id.to_string().to_kind(),
                            item_id.to_kind(),
                            (quantity + 1).to_kind(),
                        ]));
                    },
                    None => break ms
                }
            };
            tx.buffer_write(ms);
            Ok(())
        }.await;
        (tx, result)
    }).await;

    match tx_result {
        Ok(success)  => {
            Ok(warp::reply::with_status(
                warp::reply::html(format!("ts={}", match success.0 {
                    Some(s) => s.seconds,
                    None => -1
                })),
                warp::http::StatusCode::OK,
            ))
        },
        Err(e) => {
            Ok(warp::reply::with_status(
                warp::reply::html( match e {
                    Error::TxError(e)  => e.to_string(),
                    Error::ParseError(e) => e.to_string(),
                    Error::NoDataFound(e) => e.to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn read_inventory_handler(client: Arc<Client>, user_id: String) -> Result<impl Reply, Rejection> {
    let mut tx = match client.single().await {
        Ok(tx) => tx,
        Err(e) => {
            let error_message =  "aaa".to_string();
            return Ok(warp::reply::with_status(
                warp::reply::html(error_message),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };
    match read(user_id, tx.deref_mut()).await {
        Ok(inventory)  => {
            Ok(warp::reply::with_status(
                warp::reply::html(format!("user={}, item={}, character={}", inventory.0, inventory.1,inventory.2)),
                warp::http::StatusCode::OK,
            ))
        },
        Err(e) => {
            Ok(warp::reply::with_status(
                warp::reply::html( match e {
                    Error::TxError(e)  => e.to_string(),
                    Error::ParseError(e) => e.to_string(),
                    Error::NoDataFound(e) => e.to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn read(user_id:String, mut tx: &mut Transaction) -> Result<(String,usize,usize), Error> {
    let mut stmt = Statement::new("SELECT * , \
            ARRAY (SELECT AS STRUCT * FROM UserItem WHERE UserId = @Param1 ) AS UserItem, \
            ARRAY (SELECT AS STRUCT * FROM UserCharacter WHERE UserId = @Param1 ) AS UserCharacter  \
            FROM User \
            WHERE UserId = @Param1");
    stmt.add_param("Param1", user_id.to_string());
    let mut reader = tx.query(stmt).await.map_err(|e| Error::TxError(TxError::GRPC(e)))?;
    let row = match reader.next().await.map_err(|e| Error::TxError(TxError::GRPC(e)))?{
        Some(row) => row,
        None => return Err(Error::NoDataFound(user_id))
    };
    let user_id= row.column_by_name::<String>("UserId")
        .map_err(Error::ParseError)?;
    let user_items= row.column_by_name::<Vec<model::UserItem>>("UserItem")
        .map_err(Error::ParseError)?;
    let user_characters = row.column_by_name::<Vec<model::UserCharacter>>("UserCharacter")
        .map_err(Error::ParseError)?;
    Ok((
        user_id,
        user_items.len(),
        user_characters.len()
    ))
}


