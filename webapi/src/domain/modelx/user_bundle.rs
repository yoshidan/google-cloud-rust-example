use google_cloud_spanner_derive::Query;
use serde::Serialize;

use crate::domain::model::user_character::UserCharacter;
use crate::domain::model::user_item::UserItem;

#[derive(Debug, Clone, Query, Serialize)]
pub struct UserBundle {
    pub user_id: String,
    pub user_characters: Vec<UserCharacter>,
    pub user_items: Vec<UserItem>,
}
