use crate::application::usecase::user_use_case::UserUseCase;
use crate::domain::modelx::user_bundle::UserBundle;
use actix_web::HttpResponse;
use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::grpc::Status;
use google_cloud_spanner::client::TxError;
use std::sync::Arc;

pub struct UserApi {
    user_use_case: Arc<UserUseCase>,
}

impl UserApi {
    pub fn new(user_use_case: Arc<UserUseCase>) -> Self {
        Self { user_use_case }
    }

    #[tracing::instrument(skip(self,ctx))]
    pub async fn create_new_user(&self, ctx: CancellationToken) -> HttpResponse {
        match self.user_use_case.create_new_user(ctx).await {
            Ok(result) => HttpResponse::Ok().body(result),
            Err(err) => HttpResponse::InternalServerError().body(format!("{:?}", err)),
        }
    }

    #[tracing::instrument(skip(self,ctx))]
    pub async fn get_inventory(&self, ctx: CancellationToken, user_id: &str) -> HttpResponse {
        match self.user_use_case.get_inventory(ctx, user_id).await {
            Ok(result) => HttpResponse::Ok().json(format!("{:?}", result)),
            Err(err) => HttpResponse::InternalServerError().body(format!("{:?}", err)),
        }
    }


}
