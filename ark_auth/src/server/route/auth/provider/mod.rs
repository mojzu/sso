pub mod github;
pub mod local;
pub mod microsoft;

use crate::core;
use actix_web::{http::header, web, HttpResponse};
use url::Url;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope("/provider")
        .service(local::route_v1_scope())
        .service(github::route_v1_scope())
        .service(microsoft::route_v1_scope())
}

// TODO(feature): Support more OAuth2 providers.

pub fn oauth2_redirect(service: core::Service, token: core::UserToken) -> HttpResponse {
    // TODO(refactor): Add callback type to service URLs, builder method.
    let mut url = Url::parse(&service.url).unwrap();
    let query = serde_urlencoded::to_string(token).unwrap();
    url.set_query(Some(&query));

    HttpResponse::Found()
        .header(header::LOCATION, url.as_str())
        .finish()
}
