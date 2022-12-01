use crate::InjectedApi;
use actix_web::{get, post,  web, HttpResponse};
use google_cloud_gax::cancel::CancellationToken;

#[get("/api/user/{user_id}/inventory")]
pub async fn get_user_inventory(dicon: web::Data<InjectedApi>, user_id: web::Path<String>) -> HttpResponse {
    let ctx = CancellationToken::new();
    dicon.user_api.get_inventory(ctx, &user_id.into_inner()).await
}

#[post("/api/user")]
pub async fn create_new_user(dicon: web::Data<InjectedApi>) -> HttpResponse {
    let ctx = CancellationToken::new();
    dicon.user_api.create_new_user(ctx).await
}