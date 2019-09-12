mod github;
mod local;
mod microsoft;

use crate::{server_api::path, Service, UserToken};
use actix_web::{http::header, web, HttpResponse};

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::PROVIDER)
        .service(local::route_v1_scope())
        .service(github::route_v1_scope())
        .service(microsoft::route_v1_scope())
}

fn oauth2_redirect(service: Service, token: UserToken) -> HttpResponse {
    let url = service.callback_url("oauth2", token);
    HttpResponse::Found()
        .header(header::LOCATION, url.as_str())
        .finish()
}
