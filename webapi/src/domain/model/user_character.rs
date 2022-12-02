// DON'T EDIT. this code is generated by nene.
use google_cloud_googleapis::spanner::v1::Mutation;
use google_cloud_gax::grpc::Status;
use google_cloud_spanner::client::{RunInTxError, TxError};
use google_cloud_spanner::key::Key;
use google_cloud_spanner::mutation::{
    delete, insert_or_update_struct, insert_struct, replace_struct, update_struct,
};
use google_cloud_spanner::reader::AsyncIterator;
use google_cloud_spanner::row::{Error as RowError, Row, Struct, TryFromStruct};
use google_cloud_spanner::statement::{Kinds, Statement, ToKind, ToStruct, Types};
use google_cloud_spanner::transaction::Transaction;
use google_cloud_spanner::transaction::CallOptions;
use google_cloud_spanner::value::CommitTimestamp;
use google_cloud_spanner_derive::Table;
use std::convert::TryFrom;
use crate::domain::model::read_by_statement;

pub const TABLE_NAME: &str = "UserCharacter";
pub const COLUMN_USER_ID: &str = "UserId";
pub const COLUMN_CHARACTER_ID: &str = "CharacterId";
pub const COLUMN_LEVEL: &str = "Level";
pub const COLUMN_ACQUIRED_AT: &str = "AcquiredAt";
pub const COLUMN_UPDATED_AT: &str = "UpdatedAt";

#[derive(Debug,Clone,Default,Table,serde::Serialize,serde::Deserialize)]
pub struct UserCharacter {
    pub user_id: String,
    pub character_id: i64,
    pub level: i64,
    pub acquired_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserCharacter {
    pub fn insert(&self) -> Mutation {
        insert_struct(TABLE_NAME, &self)
    }

    pub fn update(&self) -> Mutation {
        update_struct(TABLE_NAME, &self)
    }

    pub fn replace(&self) -> Mutation {
        replace_struct(TABLE_NAME, &self)
    }

    pub fn insert_or_update(&self) -> Mutation {
        insert_or_update_struct(TABLE_NAME, &self)
    }

    pub fn delete(&self) -> Mutation {
        delete(TABLE_NAME, Key::composite(&[&self.user_id, &self.character_id]))
    }

    pub async fn read_by_user_id(
       tx: &mut Transaction, user_id: &str, options: Option<CallOptions>
    ) -> Result<Vec<Self>, RunInTxError> {
         let mut stmt = Statement::new("SELECT * From UserCharacter WHERE UserId = @UserId");
         stmt.add_param(COLUMN_USER_ID, &user_id);
         read_by_statement(tx, stmt, options).await
    }

    pub async fn find_by_pk(
       tx: &mut Transaction, user_id: &str, character_id: &i64, options: Option<CallOptions>
    ) -> Result<Option<Self>, RunInTxError> {
         let mut stmt = Statement::new("SELECT * From UserCharacter WHERE UserId = @UserId AND CharacterId = @CharacterId");
         stmt.add_param(COLUMN_USER_ID, &user_id);
         stmt.add_param(COLUMN_CHARACTER_ID, character_id);
         let mut rows = read_by_statement(tx, stmt, options).await?;
         if !rows.is_empty() {
            Ok(rows.pop())
         } else {
            Ok(None)
         }
    }
}
