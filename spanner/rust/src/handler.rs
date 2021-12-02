use crate::model;

use chrono::Utc;
use google_cloud_gax::invoke::TryAs;
use google_cloud_googleapis::Status;
use google_cloud_spanner::client::{Client, TxError, RunInTxError};

use google_cloud_spanner::mutation::{insert_struct, update, update_map};
use google_cloud_spanner::reader::AsyncIterator;
use google_cloud_spanner::row::Error as RowError;
use google_cloud_spanner::session::SessionError;
use google_cloud_spanner::statement::{Statement, ToKind};
use google_cloud_spanner::transaction::Transaction;

use std::ops::DerefMut;

use uuid::Uuid;
use warp::{Rejection, Reply};
use tonic::codegen::Body;
use google_cloud_spanner::value::Timestamp;
use tonic::Code;

pub async fn create_user_handler(client: Client) -> Result<impl Reply, Rejection> {
    let mut ms = vec![];
    let user_id = Uuid::new_v4().to_string();
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
            updated_at: Utc::now(),
        };
        ms.push(insert_struct("UserItem", user_item));

        let user_character = model::UserCharacter {
            user_id: user_id.to_string(),
            character_id: i,
            level: 1,
            acquired_at: Utc::now(),
            updated_at: Utc::now(),
        };
        ms.push(insert_struct("UserCharacter", user_character));
    }

    match client.apply(ms).await {
        Ok(_) => Ok(warp::reply::with_status(
            warp::reply::html(user_id.to_string()),
            warp::http::StatusCode::OK,
        )),
        Err(e) => {
            log::error!("error {:?}", e);
            Ok(warp::reply::with_status(
                warp::reply::html(match e {
                    TxError::GRPC(e) => e.to_string(),
                    TxError::InvalidSession(_e) => "session error".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn update_inventory_handler(
    client: Client,
    user_id: String,
) -> Result<impl Reply, Rejection> {
    let tx_result :Result<(Option<Timestamp>, ()), RunInTxError> = client
        .read_write_transaction(|tx| {
            let user_id = user_id.clone();
            Box::pin(async move {
                let mut stmt = Statement::new("SELECT * From UserItem WHERE UserId = @UserId");
                stmt.add_param("UserId", &user_id);
                let mut reader = tx.query(stmt).await?;
                let mut ms = vec![];
                while let Some(row) = reader.next().await? {
                    let item_id = row.column_by_name::<i64>("ItemId")?;
                    let quantity = row.column_by_name::<i64>("Quantity")? + 1;
                    ms.push(update_map("User_Item", &[(&"UserId", &user_id), (&"ItemId", &item_id), (&"Quantity", &quantity)]));
                }
                tx.buffer_write(ms);
                Ok(())
            })
        })
        .await;

    match tx_result {
        Ok(success) => Ok(warp::reply::with_status(
            warp::reply::html(format!(
                "ts={}",
                match success.0 {
                    Some(s) => s.seconds,
                    None => -1,
                }
            )),
            warp::http::StatusCode::OK,
        )),
        Err(e) => {
            log::error!("error {:?}", e);
            Ok(warp::reply::with_status(
                warp::reply::html(format!("error {:?}", e)),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn read_inventory_handler(
    client: Client,
    user_id: String,
) -> Result<impl Reply, Rejection> {
    let mut tx = match client.single().await {
        Ok(tx) => tx,
        Err(_e) => {
            let error_message = "aaa".to_string();
            return Ok(warp::reply::with_status(
                warp::reply::html(error_message),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };
    match read(user_id, tx.deref_mut()).await {
        Ok(inventory) => Ok(warp::reply::with_status(
            warp::reply::html(format!(
                "user={}, item={}, character={}",
                inventory.0, inventory.1, inventory.2
            )),
            warp::http::StatusCode::OK,
        )),
        Err(e) => {
            log::error!("error {:?}", e);
            Ok(warp::reply::with_status(
                warp::reply::html(format!("error {:?}", e)),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn read(user_id: String, tx: &mut Transaction) -> Result<(String, usize, usize), RunInTxError> {
    let mut stmt = Statement::new("SELECT * , \
            ARRAY (SELECT AS STRUCT * FROM UserItem WHERE UserId = @Param1 ) AS UserItem, \
            ARRAY (SELECT AS STRUCT * FROM UserCharacter WHERE UserId = @Param1 ) AS UserCharacter  \
            FROM User \
            WHERE UserId = @Param1");
    stmt.add_param("Param1", &user_id);
    let mut reader = tx.query(stmt).await?;
    let row = match reader.next().await? {
        Some(row) => row,
        None => return Err(RunInTxError::Any(anyhow::Error::new(tonic::Status::new(tonic::Code::NotFound, "no row found")))),
    };
    let user_id = row.column_by_name::<String>("UserId")?;
    let user_items = row.column_by_name::<Vec<model::UserItem>>("UserItem")?;
    let user_characters = row.column_by_name::<Vec<model::UserCharacter>>("UserCharacter")?;
    Ok((user_id, user_items.len(), user_characters.len()))
}
