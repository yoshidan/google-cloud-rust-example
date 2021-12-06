// DON'T EDIT. this code is generated by nene.
use google_cloud_googleapis::spanner::v1::Mutation;
use google_cloud_googleapis::Status;
use google_cloud_spanner::client::{RunInTxError, TxError};
use google_cloud_spanner::key::Key;
use google_cloud_spanner::mutation::{
    delete, insert_or_update_struct, insert_struct, replace_struct, update_struct,
};
use google_cloud_spanner::reader::AsyncIterator;
use google_cloud_spanner::row::{Error as RowError, Row, Struct, TryFromStruct};
use google_cloud_spanner::statement::{Kinds, Statement, ToKind, ToStruct, Types};
use google_cloud_spanner::transaction::Transaction;
use google_cloud_spanner::value::CommitTimestamp;
use std::convert::TryFrom;

pub const TABLE_NAME: &str = "User";
pub const COLUMN_USER_ID: &str = "UserId";
pub const COLUMN_PREMIUM: &str = "Premium";
pub const COLUMN_UPDATED_AT: &str = "UpdatedAt";

pub struct User {
    pub user_id: String,
    pub premium: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
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
        delete(TABLE_NAME, Key::key(&self.user_id))
    }

    pub async fn find_by_pk(
       tx: &mut Transaction, user_id: &String
    ) -> Result<Option<Self>, RunInTxError> {
         let mut stmt = Statement::new("SELECT * From User WHERE UserId = @UserId");
         stmt.add_param(COLUMN_USER_ID, user_id);
         let mut rows = Self::read_by_statement(tx, stmt).await?;
         if !rows.is_empty() {
            Ok(rows.pop())
         } else {
            Ok(None)
         }
    }

    pub async fn read_by_statement(
        tx: &mut Transaction,
        stmt: Statement,
    ) -> Result<Vec<Self>, RunInTxError> {
        let mut reader = tx.query(stmt).await?;
        let mut result = vec![];
        while let Some(row) = reader.next().await? {
            let data = Self::try_from(row)?;
            result.push(data)
        }
        Ok(result)
    }
}

impl ToStruct for User {
    fn to_kinds(&self) -> Kinds {
        vec![
            (COLUMN_USER_ID, self.user_id.to_kind()),
            (COLUMN_PREMIUM, self.premium.to_kind()),
            (COLUMN_UPDATED_AT, CommitTimestamp::new().to_kind()),
        ]
    }

    fn get_types() -> Types {
        vec![
            (COLUMN_USER_ID, String::get_type()),
            (COLUMN_PREMIUM, bool::get_type()),
            (COLUMN_UPDATED_AT, CommitTimestamp::get_type()),
        ]
    }
}

impl TryFromStruct for User {
    fn try_from_struct(s: Struct<'_>) -> Result<Self, RowError> {
        Ok(User {
            user_id: s.column_by_name(COLUMN_USER_ID)?,
            premium: s.column_by_name(COLUMN_PREMIUM)?,
            updated_at: s.column_by_name(COLUMN_UPDATED_AT)?,
        })
    }
}

impl TryFrom<Row> for User {
    type Error = RowError;
    fn try_from(row: Row) -> Result<Self, RowError> {
        Ok(User {
            user_id: row.column_by_name(COLUMN_USER_ID)?,
            premium: row.column_by_name(COLUMN_PREMIUM)?,
            updated_at: row.column_by_name(COLUMN_UPDATED_AT)?,
        })
    }
}
