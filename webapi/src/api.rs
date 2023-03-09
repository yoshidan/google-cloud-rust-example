use crate::InjectedApi;
use actix_web::{get, post, web, HttpResponse};

#[get("/api/user/{user_id}/inventory")]
pub async fn get_user_inventory(dicon: web::Data<InjectedApi>, user_id: web::Path<String>) -> HttpResponse {
    dicon.user_api.get_inventory(&user_id.into_inner()).await
}

#[post("/api/user")]
pub async fn create_new_user(dicon: web::Data<InjectedApi>) -> HttpResponse {
    dicon.user_api.create_new_user().await
}
