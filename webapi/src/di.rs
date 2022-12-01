use crate::application::api::user_api::UserApi;
use crate::application::usecase::user_use_case::UserUseCase;
use crate::infrastructure::repository::user_character_repository::SpannerUserCharacterRepository;
use crate::infrastructure::repository::user_item_repository::SpannerUserItemRepository;
use crate::infrastructure::repository::user_repository::SpannerUserRepository;
use google_cloud_spanner::client::Client as SpannerClient;
use std::sync::Arc;

#[derive(Clone)]
pub struct InjectedApi {
    pub user_api: Arc<UserApi>,
}

impl InjectedApi {
    pub fn new(spanner_client: SpannerClient) -> Self {
        let user_repository = Arc::new(SpannerUserRepository::new(spanner_client.clone()));
        let user_item_repository = Arc::new(SpannerUserItemRepository::new(spanner_client.clone()));
        let user_character_repository = Arc::new(SpannerUserCharacterRepository::new(spanner_client.clone()));

        let user_use_case = Arc::new(UserUseCase::new(
            spanner_client,
            user_repository,
            user_item_repository,
            user_character_repository,
        ));
        Self {
            user_api: Arc::new(UserApi::new(user_use_case)),
        }
    }
}
