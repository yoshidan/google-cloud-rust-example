use std::sync::Arc;

use actix_web::HttpResponse;

use crate::application::usecase::user_use_case::UserUseCase;

pub struct UserApi {
    user_use_case: Arc<UserUseCase>,
}

impl UserApi {
    pub fn new(user_use_case: Arc<UserUseCase>) -> Self {
        Self { user_use_case }
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_new_user(&self) -> HttpResponse {
        match self.user_use_case.create_new_user().await {
            Ok(result) => HttpResponse::Ok().body(result),
            Err(err) => HttpResponse::InternalServerError().body(format!("{:?}", err)),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_inventory(&self, user_id: &str) -> HttpResponse {
        match self.user_use_case.get_inventory(user_id).await {
            Ok(result) => HttpResponse::Ok().json(result),
            Err(err) => HttpResponse::InternalServerError().body(format!("{:?}", err)),
        }
    }
}
