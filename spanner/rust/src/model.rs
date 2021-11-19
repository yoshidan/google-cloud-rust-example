use chrono::{DateTime, Utc};
use google_cloud_spanner::row::{Error as RowError, Struct, TryFromStruct};
use google_cloud_spanner::statement::{Kinds, ToKind, ToStruct, Types};
use google_cloud_spanner::value::CommitTimestamp;

/// User
pub struct User {
    pub user_id: String,
    pub premium: bool,
    pub updated_at: DateTime<Utc>,
}

impl ToStruct for User {
    fn to_kinds(&self) -> Kinds {
        vec![
            ("UserId", self.user_id.to_kind()),
            ("Premium", self.premium.to_kind()),
            ("UpdatedAt", CommitTimestamp::new().to_kind()),
        ]
    }

    fn get_types() -> Types {
        vec![
            ("UserId", String::get_type()),
            ("Premium", bool::get_type()),
            ("UpdatedAt", CommitTimestamp::get_type()),
        ]
    }
}

impl TryFromStruct for User {
    fn try_from(s: Struct<'_>) -> Result<Self, RowError> {
        Ok(User {
            user_id: s.column_by_name("UserId")?,
            premium: s.column_by_name("Premium")?,
            updated_at: s.column_by_name("UpdatedAt")?,
        })
    }
}

/// UserItem
pub struct UserItem {
    pub user_id: String,
    pub item_id: i64,
    pub quantity: i64,
    pub updated_at: DateTime<Utc>,
}

impl ToStruct for UserItem {
    fn to_kinds(&self) -> Kinds {
        vec![
            ("UserId", self.user_id.to_kind()),
            ("ItemId", self.item_id.to_kind()),
            ("Quantity", self.quantity.to_kind()),
            ("UpdatedAt", CommitTimestamp::new().to_kind()),
        ]
    }

    fn get_types() -> Types {
        vec![
            ("UserId", String::get_type()),
            ("ItemId", i64::get_type()),
            ("Quantity", i64::get_type()),
            ("UpdatedAt", CommitTimestamp::get_type()),
        ]
    }
}

impl TryFromStruct for UserItem {
    fn try_from(s: Struct<'_>) -> Result<Self, RowError> {
        Ok(UserItem {
            user_id: s.column_by_name("UserId")?,
            item_id: s.column_by_name("ItemId")?,
            quantity: s.column_by_name("Quantity")?,
            updated_at: s.column_by_name("UpdatedAt")?,
        })
    }
}

/// UserCharacter
pub struct UserCharacter {
    pub user_id: String,
    pub character_id: i64,
    pub level: i64,
    pub acquired_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ToStruct for UserCharacter {
    fn to_kinds(&self) -> Kinds {
        vec![
            ("UserId", self.user_id.to_kind()),
            ("CharacterId", self.character_id.to_kind()),
            ("Level", self.level.to_kind()),
            ("AcquiredAt", self.acquired_at.to_kind()),
            ("UpdatedAt", CommitTimestamp::new().to_kind()),
        ]
    }

    fn get_types() -> Types {
        vec![
            ("UserId", String::get_type()),
            ("CharacterId", i64::get_type()),
            ("Level", i64::get_type()),
            ("AcquiredAt", DateTime::<Utc>::get_type()),
            ("UpdatedAt", CommitTimestamp::get_type()),
        ]
    }
}

impl TryFromStruct for UserCharacter {
    fn try_from(s: Struct<'_>) -> Result<Self, RowError> {
        Ok(UserCharacter {
            user_id: s.column_by_name("UserId")?,
            character_id: s.column_by_name("CharacterId")?,
            level: s.column_by_name("Level")?,
            acquired_at: s.column_by_name("AcquiredAt")?,
            updated_at: s.column_by_name("UpdatedAt")?,
        })
    }
}
