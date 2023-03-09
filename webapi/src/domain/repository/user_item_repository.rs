use async_trait::async_trait;

use crate::domain::model::user_item::UserItem;
use google_cloud_spanner::client::Error;
use google_cloud_spanner::transaction::Transaction;
use google_cloud_spanner::transaction_rw::ReadWriteTransaction;

#[async_trait]
pub trait UserItemRepository {
    async fn find_by_pk(
        &self,
        tx: Option<&mut Transaction>,
        user_id: &str,
        user_card_number: i64,
    ) -> Result<Option<UserItem>, Error>;
    async fn insert(&self, tx: Option<&mut ReadWriteTransaction>, target: &UserItem) -> Result<(), Error>;
}
