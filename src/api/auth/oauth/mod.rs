pub mod github;
pub mod microsoft;

use crate::api::{auth, ApiData, ApiError};
use crate::models::AuthService;
use actix_web::http::{header, StatusCode};
use actix_web::{web, HttpResponse};
use url::Url;

/// Version 1 authentication oauth routes scope.
pub fn v1_service() -> actix_web::Scope {
    web::scope("/oauth")
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlResponse {
    pub url: String,
}

pub fn oauth_login(
    data: &web::Data<ApiData>,
    email: &str,
    service_id: i64,
) -> Result<(auth::TokenResponse, AuthService), ApiError> {
    let token = data
        .db
        .oauth_login(email, service_id)
        .map_err(ApiError::Db)?;
    let service = data
        .db
        .service_read_by_id(service_id, service_id)
        .map_err(ApiError::Db)?;
    Ok((token, service))
}

pub fn oauth_redirect(token: auth::TokenResponse, service: AuthService) -> HttpResponse {
    let mut url = Url::parse(&service.service_url).unwrap();
    let token_query = format!("token={}", token.token);
    url.set_query(Some(&token_query));

    HttpResponse::build(StatusCode::FOUND)
        .header(header::LOCATION, url.as_str())
        .finish()
}
