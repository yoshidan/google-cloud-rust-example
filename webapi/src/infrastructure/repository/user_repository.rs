use crate::domain::modelx::user_bundle::UserBundle;
use crate::domain::repository::user_repository::UserRepository;
use async_trait::async_trait;
use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_googleapis::spanner::v1::execute_sql_request::QueryMode;
use google_cloud_spanner::client::{Client, ReadWriteTransactionOption, RunInTxError, TxError};
use google_cloud_spanner::reader::AsyncIterator;
use google_cloud_spanner::statement::Statement;
use google_cloud_spanner::transaction::{CallOptions, QueryOptions, Transaction};
use google_cloud_spanner::transaction_rw::ReadWriteTransaction;
use crate::domain::model::user::User;
use crate::lib::context::Context;

pub struct SpannerUserRepository {
    client: Client,
}

impl SpannerUserRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UserRepository for SpannerUserRepository {
    async fn find_by_pk(&self, ctx: &mut Context, tx: Option<&mut Transaction>, user_id: &str) -> Result<UserBundle, RunInTxError> {
        let sql = "
SELECT
    UserId,
	ARRAY(SELECT AS STRUCT * FROM UserItem WHERE UserId = @UserId) AS UserItems,
	ARRAY(SELECT AS STRUCT * FROM UserCharacter WHERE UserId = @UserId) AS UserCharacters
FROM User WHERE UserID = @UserID
";
        let mut statement = Statement::new(sql);
        statement.add_param("UserID", &user_id);
        let row = match tx {
            Some(tx) => tx.query_with_option(statement, ctx.into()).await?.next().await?,
            None => self.client.single().await?.query_with_option(statement, ctx.into()).await?.next().await?
        };

        match row {
            Some(row) => Ok(row.try_into()?),
            None => Err(Status::new(Code::NotFound, "user not found").into()),
        }
    }

    async fn insert(&self, ctx: &mut Context, tx: Option<&mut ReadWriteTransaction>, target: &User) -> Result<(), RunInTxError> {
        match tx {
            Some(tx) => tx.buffer_write(vec![target.insert()]),
            None => {
                let _ = self.client.apply_with_option(vec![target.insert()], ctx.into()).await?;
            }
        };
        Ok(())
    }
}
