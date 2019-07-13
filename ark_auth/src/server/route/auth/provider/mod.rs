pub mod github;
pub mod local;
pub mod microsoft;

use crate::core;
use actix_web::{http::header, web, HttpResponse};

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope("/provider")
        .service(local::route_v1_scope())
        .service(github::route_v1_scope())
        .service(microsoft::route_v1_scope())
}

pub fn oauth2_redirect(service: core::Service, token: core::UserToken) -> HttpResponse {
    let url = service.callback_url("oauth2", token);
    HttpResponse::Found()
        .header(header::LOCATION, url.as_str())
        .finish()
}
