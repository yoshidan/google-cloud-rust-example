use crate::domain::model::user_character::UserCharacter;
use crate::domain::model::user_item::UserItem;
use google_cloud_spanner_derive::Query;

#[derive(Debug, Clone, Query)]
pub struct UserBundle {
    pub user_id: String,
    pub user_characters: Vec<UserCharacter>,
    pub user_items: Vec<UserItem>,
}