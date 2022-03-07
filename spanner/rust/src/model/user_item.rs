// DON'T EDIT. this code is generated by nene.
use google_cloud_googleapis::spanner::v1::Mutation;
use google_cloud_googleapis::Status;
use google_cloud_spanner::client::{RunInTxError, TxError};
use google_cloud_spanner::key::Key;
use google_cloud_spanner::mutation::{delete, insert_or_update_struct, insert_struct, replace_struct, update_struct};
use google_cloud_spanner::reader::AsyncIterator;
use google_cloud_spanner::row::{Error as RowError, Row, Struct, TryFromStruct};
use google_cloud_spanner::statement::{Kinds, Statement, ToKind, ToStruct, Types};
use google_cloud_spanner::transaction::Transaction;
use google_cloud_spanner::value::CommitTimestamp;
use std::convert::TryFrom;

pub const TABLE_NAME: &str = "UserItem";
pub const COLUMN_USER_ID: &str = "UserId";
pub const COLUMN_ITEM_ID: &str = "ItemId";
pub const COLUMN_QUANTITY: &str = "Quantity";
pub const COLUMN_UPDATED_AT: &str = "UpdatedAt";

pub struct UserItem {
    pub user_id: String,
    pub item_id: i64,
    pub quantity: i64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserItem {
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
        delete(TABLE_NAME, Key::composite(&[&self.user_id, &self.item_id]))
    }

    pub async fn read_by_user_id(tx: &mut Transaction, user_id: &String) -> Result<Vec<Self>, RunInTxError> {
        let mut stmt = Statement::new("SELECT * From UserItem WHERE UserId = @UserId");
        stmt.add_param(COLUMN_USER_ID, user_id);
        Self::read_by_statement(tx, stmt).await
    }

    pub async fn find_by_pk(
        tx: &mut Transaction,
        user_id: &String,
        item_id: &i64,
    ) -> Result<Option<Self>, RunInTxError> {
        let mut stmt = Statement::new("SELECT * From UserItem WHERE UserId = @UserId AND ItemId = @ItemId");
        stmt.add_param(COLUMN_USER_ID, user_id);
        stmt.add_param(COLUMN_ITEM_ID, item_id);
        let mut rows = Self::read_by_statement(tx, stmt).await?;
        if !rows.is_empty() {
            Ok(rows.pop())
        } else {
            Ok(None)
        }
    }

    pub async fn read_by_statement(tx: &mut Transaction, stmt: Statement) -> Result<Vec<Self>, RunInTxError> {
        let mut reader = tx.query(stmt).await?;
        let mut result = vec![];
        while let Some(row) = reader.next().await? {
            let data = Self::try_from(row)?;
            result.push(data)
        }
        Ok(result)
    }
}

impl ToStruct for UserItem {
    fn to_kinds(&self) -> Kinds {
        vec![
            (COLUMN_USER_ID, self.user_id.to_kind()),
            (COLUMN_ITEM_ID, self.item_id.to_kind()),
            (COLUMN_QUANTITY, self.quantity.to_kind()),
            (COLUMN_UPDATED_AT, CommitTimestamp::new().to_kind()),
        ]
    }

    fn get_types() -> Types {
        vec![
            (COLUMN_USER_ID, String::get_type()),
            (COLUMN_ITEM_ID, i64::get_type()),
            (COLUMN_QUANTITY, i64::get_type()),
            (COLUMN_UPDATED_AT, CommitTimestamp::get_type()),
        ]
    }
}

impl TryFromStruct for UserItem {
    fn try_from_struct(s: Struct<'_>) -> Result<Self, RowError> {
        Ok(UserItem {
            user_id: s.column_by_name(COLUMN_USER_ID)?,
            item_id: s.column_by_name(COLUMN_ITEM_ID)?,
            quantity: s.column_by_name(COLUMN_QUANTITY)?,
            updated_at: s.column_by_name(COLUMN_UPDATED_AT)?,
        })
    }
}

impl TryFrom<Row> for UserItem {
    type Error = RowError;
    fn try_from(row: Row) -> Result<Self, RowError> {
        Ok(UserItem {
            user_id: row.column_by_name(COLUMN_USER_ID)?,
            item_id: row.column_by_name(COLUMN_ITEM_ID)?,
            quantity: row.column_by_name(COLUMN_QUANTITY)?,
            updated_at: row.column_by_name(COLUMN_UPDATED_AT)?,
        })
    }
}