use google_cloud_spanner::client::{Client, Error};
use std::sync::Arc;

use crate::domain::model::user::User;
use crate::domain::model::user_character::UserCharacter;
use crate::domain::model::user_item::UserItem;
use crate::domain::modelx::user_bundle::UserBundle;
use crate::domain::repository::user_character_repository::UserCharacterRepository;
use crate::domain::repository::user_item_repository::UserItemRepository;
use crate::domain::repository::user_repository::UserRepository;

pub struct UserUseCase {
    transactor: Client,
    user_repository: Arc<dyn UserRepository + 'static + Send + Sync>,
    user_item_repository: Arc<dyn UserItemRepository + 'static + Send + Sync>,
    user_character_repository: Arc<dyn UserCharacterRepository + 'static + Send + Sync>,
}

impl UserUseCase {
    pub fn new(
        transactor: Client,
        user_repository: Arc<dyn UserRepository + 'static + Send + Sync>,
        user_item_repository: Arc<dyn UserItemRepository + 'static + Send + Sync>,
        user_character_repository: Arc<dyn UserCharacterRepository + 'static + Send + Sync>,
    ) -> Self {
        Self {
            transactor,
            user_repository,
            user_item_repository,
            user_character_repository,
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn create_new_user(&self) -> Result<String, anyhow::Error> {
        let result: Result<(_, String), Error> = self
            .transactor
            .read_write_transaction(|tx| {
                let user_repository = self.user_repository.clone();
                let user_item_repository = self.user_item_repository.clone();
                let user_character_repository = self.user_character_repository.clone();
                Box::pin(async move {
                    let user_id = uuid::Uuid::new_v4().to_string();

                    user_repository
                        .insert(
                            Some(tx),
                            &User {
                                user_id: user_id.clone(),
                                ..Default::default()
                            },
                        )
                        .await?;

                    user_item_repository
                        .insert(
                            Some(tx),
                            &UserItem {
                                user_id: user_id.clone(),
                                item_id: 1,
                                quantity: 10,
                                ..Default::default()
                            },
                        )
                        .await?;

                    for i in 1..11 {
                        user_character_repository
                            .insert(
                                Some(tx),
                                &UserCharacter {
                                    user_id: user_id.clone(),
                                    character_id: i,
                                    level: 1,
                                    ..Default::default()
                                },
                            )
                            .await?;
                    }
                    Ok(user_id)
                })
            })
            .await;

        Ok(result?.1)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_inventory(&self, user_id: &str) -> Result<UserBundle, anyhow::Error> {
        let user_bundle = self.user_repository.find_by_pk(None, user_id).await?;
        Ok(user_bundle)
    }
}
