use crate::model;

use chrono::Utc;

use google_cloud_spanner::client::{Client, RunInTxError, TxError};

use google_cloud_spanner::reader::AsyncIterator;

use google_cloud_spanner::statement::Statement;
use google_cloud_spanner::transaction::Transaction;

use std::ops::DerefMut;

use uuid::Uuid;
use warp::{Rejection, Reply};

use google_cloud_spanner::value::Timestamp;

use std::convert::TryFrom;
use google_cloud_spanner::transaction_rw::ReadWriteTransaction;
use tracing::instrument;

#[instrument(skip(client))]
pub async fn create_user_handler(client: Client) -> Result<impl Reply, Rejection> {
    tracing::info!("request: create_user_handler");
    let mut ms = vec![];
    let user_id = Uuid::new_v4().to_string();
    let user = model::user::User {
        user_id: user_id.to_string(),
        premium: true,
        updated_at: Utc::now(),
    };
    ms.push(user.insert());
    for i in 0..10 {
        let user_item = model::user_item::UserItem {
            user_id: user_id.to_string(),
            item_id: i,
            quantity: 0,
            updated_at: Utc::now(),
        };
        ms.push(user_item.insert());

        let user_character = model::user_character::UserCharacter {
            user_id: user_id.to_string(),
            character_id: i,
            level: 1,
            acquired_at: Utc::now(),
            updated_at: Utc::now(),
        };
        ms.push(user_character.insert());
    }

    match client.apply(ms).await {
        Ok(_) => Ok(warp::reply::with_status(
            warp::reply::html(user_id.to_string()),
            warp::http::StatusCode::OK,
        )),
        Err(e) => {
            tracing::error!("error {:?}", e);
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

#[instrument(skip(client))]
pub async fn update_inventory_handler(client: Client, user_id: String) -> Result<impl Reply, Rejection> {
    tracing::info!("request: update_inventory_handler");
    let tx_result: Result<(Option<Timestamp>, ()), RunInTxError> = client
        .read_write_transaction(|tx, _| {
            let user_id = user_id.clone();
            Box::pin(async move {
                update_inventory(tx, user_id).await
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
            tracing::error!("error {:?}", e);
            Ok(warp::reply::with_status(
                warp::reply::html(format!("error {:?}", e)),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[instrument(skip(tx))]
async fn update_inventory(tx: &mut ReadWriteTransaction, user_id: String) -> Result<(), RunInTxError> {
    tracing::info!("update_inventory");
    let mut items = model::user_item::UserItem::read_by_user_id(tx.deref_mut(), &user_id, None).await?;
    let mut ms = vec![];
    for mut item in items.into_iter() {
        item.quantity += 1;
        ms.push(item.update());
    }
    tx.buffer_write(ms);
    Ok(())
}

#[instrument(skip(client))]
pub async fn read_inventory_handler(client: Client, user_id: String) -> Result<impl Reply, Rejection> {
    tracing::info!("request: read_inventory_handler");
    let mut tx = match client.single().await {
        Ok(tx) => tx,
        Err(e) => {
            return Ok(warp::reply::with_status(
                warp::reply::html(format!("error {:?}", e)),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };
    match read_inventory(user_id, tx.deref_mut()).await {
        Ok(inventory) => Ok(warp::reply::with_status(
            warp::reply::html(format!("user={}, item={}, character={}", inventory.0, inventory.1, inventory.2)),
            warp::http::StatusCode::OK,
        )),
        Err(e) => {
            tracing::error!("error {:?}", e);
            Ok(warp::reply::with_status(
                warp::reply::html(format!("error {:?}", e)),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[instrument(skip(tx))]
async fn read_inventory(user_id: String, tx: &mut Transaction) -> Result<(String, usize, usize), RunInTxError> {
    tracing::info!("read_inventory");
    let mut stmt = Statement::new(
        "SELECT * , \
            ARRAY (SELECT AS STRUCT * FROM UserItem WHERE UserId = @Param1 ) AS UserItem, \
            ARRAY (SELECT AS STRUCT * FROM UserCharacter WHERE UserId = @Param1 ) AS UserCharacter  \
            FROM User \
            WHERE UserId = @Param1",
    );
    stmt.add_param("Param1", &user_id);
    let mut reader = tx.query(stmt).await?;
    let row = match reader.next().await? {
        Some(row) => row,
        None => {
            return Err(RunInTxError::Any(anyhow::Error::new(tonic::Status::new(
                tonic::Code::NotFound,
                "no row found",
            ))))
        }
    };
    let user_id = row.column_by_name::<String>("UserId")?;
    let user_items = row.column_by_name::<Vec<model::user_item::UserItem>>("UserItem")?;
    let user_characters = row.column_by_name::<Vec<model::user_character::UserCharacter>>("UserCharacter")?;
    Ok((user_id, user_items.len(), user_characters.len()))
}
