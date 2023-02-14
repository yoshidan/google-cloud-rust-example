use async_trait::async_trait;

use google_cloud_spanner::client::{Client, Error};

use crate::domain::model::user_item::UserItem;
use crate::domain::repository::user_item_repository::UserItemRepository;
use crate::lib::context::Context;
use google_cloud_spanner::transaction::Transaction;
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
        ctx: &mut Context,
        tx: Option<&mut Transaction>,
        user_id: &str,
        item_id: i64,
    ) -> Result<Option<UserItem>, Error> {
        match tx {
            Some(tx) => UserItem::find_by_pk(tx, user_id, &item_id, Some(ctx.into())).await,
            None => {
                let tx = &mut self.client.single().await?;
                UserItem::find_by_pk(tx, user_id, &item_id, Some(ctx.into())).await
            }
        }
    }

    async fn insert(
        &self,
        ctx: &mut Context,
        tx: Option<&mut ReadWriteTransaction>,
        target: &UserItem,
    ) -> Result<(), Error> {
        match tx {
            Some(tx) => tx.buffer_write(vec![target.insert()]),
            None => {
                let _ = self.client.apply_with_option(vec![target.insert()], ctx.into()).await?;
            }
        };
        Ok(())
    }
}
