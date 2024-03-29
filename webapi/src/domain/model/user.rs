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

pub const TABLE_NAME: &str = "User";
pub const COLUMN_USER_ID: &str = "UserId";
pub const COLUMN_PREMIUM: &str = "Premium";
pub const COLUMN_VALUE: &str = "Value";
pub const COLUMN_UPDATED_AT: &str = "UpdatedAt";

#[derive(Debug, Clone, Table, serde::Serialize, serde::Deserialize)]
pub struct User {
    #[spanner(name = "UserId")]
    pub user_id: String,
    #[spanner(name = "Premium")]
    pub premium: bool,
    #[spanner(name = "Value")]
    pub value: google_cloud_spanner::value::SpannerNumeric,
    #[serde(with = "time::serde::rfc3339")]
    #[spanner(name = "UpdatedAt", commitTimestamp)]
    pub updated_at: time::OffsetDateTime,
}

impl Default for User {
    fn default() -> Self {
        Self {
            user_id: Default::default(),
            premium: Default::default(),
            value: Default::default(),
            updated_at: time::OffsetDateTime::now_utc(),
        }
    }
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
        delete(TABLE_NAME, Key::new(&self.user_id))
    }

    pub async fn find_by_pk(
        tx: &mut Transaction,
        user_id: &str,
        options: Option<CallOptions>,
    ) -> Result<Option<Self>, Error> {
        let mut stmt = Statement::new("SELECT * From User WHERE UserId = @UserId");
        stmt.add_param(COLUMN_USER_ID, &user_id);
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
