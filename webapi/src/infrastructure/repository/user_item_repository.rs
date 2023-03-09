use async_trait::async_trait;

use google_cloud_spanner::client::{Client, Error, ReadWriteTransactionOption};

use crate::domain::model::user_item::UserItem;
use crate::domain::repository::user_item_repository::UserItemRepository;
use google_cloud_spanner::transaction::{CallOptions, Transaction};
use google_cloud_spanner::transaction_rw::ReadWriteTransaction;

pub struct SpannerUserItemRepository {
    client: Client,
}

impl SpannerUserItemRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UserItemRepository for SpannerUserItemRepository {
    async fn find_by_pk(
        &self,
        tx: Option<&mut Transaction>,
        user_id: &str,
        item_id: i64,
    ) -> Result<Option<UserItem>, Error> {
        match tx {
            Some(tx) => UserItem::find_by_pk(tx, user_id, &item_id, Some(CallOptions::default())).await,
            None => {
                let tx = &mut self.client.single().await?;
                UserItem::find_by_pk(tx, user_id, &item_id, Some(CallOptions::default())).await
            }
        }
    }

    async fn insert(&self, tx: Option<&mut ReadWriteTransaction>, target: &UserItem) -> Result<(), Error> {
        match tx {
            Some(tx) => tx.buffer_write(vec![target.insert()]),
            None => {
                let _ = self
                    .client
                    .apply_with_option(vec![target.insert()], ReadWriteTransactionOption::default())
                    .await?;
            }
        };
        Ok(())
    }
}
