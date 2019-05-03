pub mod github;
pub mod microsoft;

use crate::core;
use crate::server::validate_token;
use actix_web::{http::header, web, HttpResponse};
use url::Url;
use validator::Validate;

// TODO(feature): Other OAuth2 providers support.

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CallbackQuery {
    #[validate(custom = "validate_token")]
    pub code: String,
    #[validate(custom = "validate_token")]
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlResponse {
    pub url: String,
}

pub fn oauth2_redirect(token: core::UserToken, service: core::Service) -> HttpResponse {
    let mut url = Url::parse(&service.service_url).unwrap();
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
                .route(web::post().to_async(github::v1))
                .route(web::get().to_async(github::v1_callback)),
        )
        .service(
            web::resource("/microsoft")
                .route(web::post().to_async(microsoft::v1))
                .route(web::get().to_async(microsoft::v1_callback)),
        )
}
