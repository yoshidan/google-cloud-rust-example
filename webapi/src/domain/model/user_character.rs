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

pub const TABLE_NAME: &str = "UserCharacter";
pub const COLUMN_USER_ID: &str = "UserId";
pub const COLUMN_CHARACTER_ID: &str = "CharacterId";
pub const COLUMN_LEVEL: &str = "Level";
pub const COLUMN_ACQUIRED_AT: &str = "AcquiredAt";
pub const COLUMN_UPDATED_AT: &str = "UpdatedAt";

#[derive(Debug, Clone, Table, serde::Serialize, serde::Deserialize)]
pub struct UserCharacter {
    #[spanner(name = "UserId")]
    pub user_id: String,
    #[spanner(name = "CharacterId")]
    pub character_id: i64,
    #[spanner(name = "Level")]
    pub level: i64,
    #[serde(with = "time::serde::rfc3339")]
    #[spanner(name = "AcquiredAt")]
    pub acquired_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    #[spanner(name = "UpdatedAt", commitTimestamp)]
    pub updated_at: time::OffsetDateTime,
}

impl Default for UserCharacter {
    fn default() -> Self {
        Self {
            user_id: Default::default(),
            character_id: Default::default(),
            level: Default::default(),
            acquired_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
        }
    }
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
        tx: &mut Transaction,
        user_id: &str,
        options: Option<CallOptions>,
    ) -> Result<Vec<Self>, Error> {
        let mut stmt = Statement::new("SELECT * From UserCharacter WHERE UserId = @UserId");
        stmt.add_param(COLUMN_USER_ID, &user_id);
        read_by_statement(tx, stmt, options).await
    }

    pub async fn find_by_pk(
        tx: &mut Transaction,
        user_id: &str,
        character_id: &i64,
        options: Option<CallOptions>,
    ) -> Result<Option<Self>, Error> {
        let mut stmt =
            Statement::new("SELECT * From UserCharacter WHERE UserId = @UserId AND CharacterId = @CharacterId");
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
