pub mod github;
pub mod microsoft;

use crate::{core, server::validate};
use actix_web::{http::header, web, HttpResponse};
use url::Url;
use validator::Validate;

// TODO(feature): Support more OAuth2 providers.

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Oauth2CallbackQuery {
    #[validate(custom = "validate::token")]
    code: String,
    #[validate(custom = "validate::token")]
    state: String,
}

impl validate::FromJsonValue<Oauth2CallbackQuery> for Oauth2CallbackQuery {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Oauth2UrlResponse {
    pub url: String,
}

pub fn oauth2_redirect(service: core::Service, token: core::UserToken) -> HttpResponse {
    let mut url = Url::parse(&service.url).unwrap();
    let token_query = format!("token={}", token.token);
    url.set_query(Some(&token_query));

    HttpResponse::Found()
        .header(header::LOCATION, url.as_str())
        .finish()
}

/// Version 1 API authentication provider scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/provider")
        .service(github::api_v1_scope())
        .service(microsoft::api_v1_scope())
}
