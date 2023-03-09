use std::convert::TryFrom;

// DON'T EDIT. this code is generated by nene.
use google_cloud_googleapis::spanner::v1::Mutation;
use google_cloud_spanner::client::Error;
use google_cloud_spanner::key::Key;
use google_cloud_spanner::mutation::{delete, insert_or_update_struct, insert_struct, replace_struct, update_struct};
use google_cloud_spanner::reader::AsyncIterator;
use google_cloud_spanner::row::{Error as RowError, Row};
use google_cloud_spanner::statement::Statement;
use google_cloud_spanner::transaction::CallOptions;
use google_cloud_spanner::transaction::Transaction;
use google_cloud_spanner_derive::Table;

pub const TABLE_NAME: &str = "UserItem";
pub const COLUMN_USER_ID: &str = "UserId";
pub const COLUMN_ITEM_ID: &str = "ItemId";
pub const COLUMN_QUANTITY: &str = "Quantity";
pub const COLUMN_UPDATED_AT: &str = "UpdatedAt";

#[derive(Debug, Clone, Table, serde::Serialize, serde::Deserialize)]
pub struct UserItem {
    #[spanner(name = "UserId")]
    pub user_id: String,
    #[spanner(name = "ItemId")]
    pub item_id: i64,
    #[spanner(name = "Quantity")]
    pub quantity: i64,
    #[serde(with = "time::serde::rfc3339")]
    #[spanner(name = "UpdatedAt", commitTimestamp)]
    pub updated_at: time::OffsetDateTime,
}

impl Default for UserItem {
    fn default() -> Self {
        Self {
            user_id: Default::default(),
            item_id: Default::default(),
            quantity: Default::default(),
            updated_at: time::OffsetDateTime::now_utc(),
        }
    }
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

    pub async fn read_by_user_id(
        tx: &mut Transaction,
        user_id: &str,
        options: Option<CallOptions>,
    ) -> Result<Vec<Self>, Error> {
        let mut stmt = Statement::new("SELECT * From UserItem WHERE UserId = @UserId");
        stmt.add_param(COLUMN_USER_ID, &user_id);
        read_by_statement(tx, stmt, options).await
    }

    pub async fn find_by_pk(
        tx: &mut Transaction,
        user_id: &str,
        item_id: &i64,
        options: Option<CallOptions>,
    ) -> Result<Option<Self>, Error> {
        let mut stmt = Statement::new("SELECT * From UserItem WHERE UserId = @UserId AND ItemId = @ItemId");
        stmt.add_param(COLUMN_USER_ID, &user_id);
        stmt.add_param(COLUMN_ITEM_ID, item_id);
        let mut rows = read_by_statement(tx, stmt, options).await?;
        if !rows.is_empty() {
            Ok(rows.pop())
        } else {
            Ok(None)
        }
    }
}

async fn read_by_statement<T: TryFrom<Row, Error = RowError>>(
    tx: &mut Transaction,
    stmt: Statement,
    options: Option<CallOptions>,
) -> Result<Vec<T>, Error> {
    let mut reader = tx.query(stmt).await?;
    if options.is_some() {
        reader.set_call_options(options.unwrap());
    }
    let mut result = vec![];
    while let Some(row) = reader.next().await? {
        result.push(row.try_into()?);
    }
    Ok(result)
}
