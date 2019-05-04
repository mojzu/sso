pub mod github;
pub mod microsoft;

use crate::core;
use crate::server::{validate_token, ValidateFromValue};
use actix_web::{http::header, web, HttpResponse};
use url::Url;
use validator::Validate;

// TODO(feature): Support more OAuth2 providers (see oauth2 documentation).

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CallbackQuery {
    #[validate(custom = "validate_token")]
    code: String,
    #[validate(custom = "validate_token")]
    state: String,
}

impl ValidateFromValue<CallbackQuery> for CallbackQuery {}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlResponse {
    url: String,
}

pub fn oauth2_redirect(service: core::Service, token: core::UserToken) -> HttpResponse {
    let mut url = Url::parse(&service.url).unwrap();
    let token_query = format!("token={}", token.token);
    url.set_query(Some(&token_query));

    HttpResponse::Found()
        .header(header::LOCATION, url.as_str())
        .finish()
}

/// Version 1 API authentication oauth2 scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/oauth2")
        .service(
            web::resource("/github")
                .route(web::post().to_async(github::request_handler))
                .route(web::get().to_async(github::callback_handler)),
        )
        .service(
            web::resource("/microsoft")
                .route(web::post().to_async(microsoft::request_handler))
                .route(web::get().to_async(microsoft::callback_handler)),
        )
}
