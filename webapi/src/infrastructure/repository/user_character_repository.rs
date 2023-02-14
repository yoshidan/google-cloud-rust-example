use async_trait::async_trait;

use google_cloud_spanner::client::{Client, Error};

use crate::domain::model::user_character::UserCharacter;
use crate::domain::repository::user_character_repository::UserCharacterRepository;
use crate::lib::context::Context;
use google_cloud_spanner::transaction::Transaction;
use google_cloud_spanner::transaction_rw::ReadWriteTransaction;

pub struct SpannerUserCharacterRepository {
    client: Client,
}

impl SpannerUserCharacterRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UserCharacterRepository for SpannerUserCharacterRepository {
    async fn find_by_pk(
        &self,
        ctx: &mut Context,
        tx: Option<&mut Transaction>,
        user_id: &str,
        item_id: i64,
    ) -> Result<Option<UserCharacter>, Error> {
        match tx {
            Some(tx) => UserCharacter::find_by_pk(tx, user_id, &item_id, Some(ctx.into())).await,
            None => {
                let tx = &mut self.client.single().await?;
                UserCharacter::find_by_pk(tx, user_id, &item_id, Some(ctx.into())).await
            }
        }
    }

    async fn insert(
        &self,
        ctx: &mut Context,
        tx: Option<&mut ReadWriteTransaction>,
        target: &UserCharacter,
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
