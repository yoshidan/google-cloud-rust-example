use async_trait::async_trait;

use crate::domain::model::user_character::UserCharacter;
use google_cloud_spanner::client::RunInTxError;
use google_cloud_spanner::transaction::Transaction;
use google_cloud_spanner::transaction_rw::ReadWriteTransaction;

use crate::lib::context::Context;

#[async_trait]
pub trait UserCharacterRepository {
    async fn find_by_pk(
        &self,
        ctx: &mut Context,
        tx: Option<&mut Transaction>,
        user_id: &str,
        user_card_number: i64,
    ) -> Result<Option<UserCharacter>, RunInTxError>;
    async fn insert(
        &self,
        ctx: &mut Context,
        tx: Option<&mut ReadWriteTransaction>,
        target: &UserCharacter,
    ) -> Result<(), RunInTxError>;
}
