use async_trait::async_trait;

use crate::domain::model::user_item::UserItem;
use crate::lib::context::Context;
use google_cloud_spanner::client::RunInTxError;
use google_cloud_spanner::transaction::Transaction;
use google_cloud_spanner::transaction_rw::ReadWriteTransaction;

#[async_trait]
pub trait UserItemRepository {
    async fn find_by_pk(
        &self,
        ctx: &mut Context,
        tx: Option<&mut Transaction>,
        user_id: &str,
        user_card_number: i64,
    ) -> Result<Option<UserItem>, RunInTxError>;
    async fn insert(
        &self,
        ctx: &mut Context,
        tx: Option<&mut ReadWriteTransaction>,
        target: &UserItem,
    ) -> Result<(), RunInTxError>;
}
