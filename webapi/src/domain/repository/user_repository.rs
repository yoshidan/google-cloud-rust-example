use crate::domain::modelx::user_bundle::UserBundle;
use async_trait::async_trait;

use crate::domain::model::user::User;
use google_cloud_spanner::client::Error;
use google_cloud_spanner::transaction::Transaction;
use google_cloud_spanner::transaction_rw::ReadWriteTransaction;

#[async_trait]
pub trait UserRepository {
    async fn find_by_pk(&self, tx: Option<&mut Transaction>, user_id: &str) -> Result<UserBundle, Error>;
    async fn insert(&self, tx: Option<&mut ReadWriteTransaction>, target: &User) -> Result<(), Error>;
}
