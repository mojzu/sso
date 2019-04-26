pub mod key;
pub mod oauth2;
pub mod reset;
pub mod token;

use crate::api::{authenticate, body_json_config, ApiData, ApiError, BodyFromValue};
use crate::db::DbError;
use crate::models::AuthService;
use actix_http::http::header::ContentType;
use actix_web::http::{header, StatusCode};
use actix_web::{middleware::identity::Identity, web, Error, HttpResponse, ResponseError};
use futures::{future, Future};
use validator::{Validate, ValidationError};

/// Version 1 authentication routes scope.
pub fn v1_service() -> actix_web::Scope {
    web::scope("/auth")
        .service(
            web::resource("/login").route(web::post().data(body_json_config()).to_async(v1_login)),
        )
        .service(reset::v1_service())
        .service(token::v1_service())
        .service(key::v1_service())
        .service(oauth2::v1_service())
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.is_empty() || password.len() > 100 {
        Err(ValidationError::new("invalid_password"))
    } else {
        Ok(())
    }
}

pub fn validate_token(token: &str) -> Result<(), ValidationError> {
    if token.is_empty() || token.len() > 512 {
        Err(ValidationError::new("invalid_token"))
    } else {
        Ok(())
    }
}

pub fn validate_key(key: &str) -> Result<(), ValidationError> {
    if key.is_empty() || key.len() > 32 {
        Err(ValidationError::new("invalid_key"))
    } else {
        Ok(())
    }
}

pub fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() || name.len() > 100 {
        Err(ValidationError::new("invalid_name"))
    } else {
        Ok(())
    }
}

pub fn validate_id(id: i64) -> Result<(), ValidationError> {
    if id < 1 {
        Err(ValidationError::new("invalid_id"))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct LoginBody {
    #[validate(email)]
    pub email: String,
    #[validate(custom = "validate_password")]
    pub password: String,
}

impl BodyFromValue<LoginBody> for LoginBody {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: i64,
    pub password_pwned: bool,
    pub token: String,
    pub token_expires: usize,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct TokenBody {
    #[validate(custom = "validate_token")]
    pub token: String,
}

impl BodyFromValue<TokenBody> for TokenBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub user_id: i64,
    pub token: String,
    pub token_expires: usize,
}

impl From<LoginResponse> for TokenResponse {
    fn from(r: LoginResponse) -> Self {
        TokenResponse {
            user_id: r.user_id,
            token: r.token,
            token_expires: r.token_expires,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyBody {
    #[validate(custom = "validate_key")]
    pub key: String,
}

impl BodyFromValue<KeyBody> for KeyBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyResponse {
    pub user_id: i64,
    pub key: String,
}

pub fn v1_login(
    data: web::Data<ApiData>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    LoginBody::from_value(body.into_inner())
        .and_then(move |body| login_inner(data, id, body))
        .then(|r| match r {
            Ok(r) => future::ok(HttpResponse::Ok().json(r)),
            Err(e) => future::ok(e.error_response()),
        })
}

fn login_inner(
    data: web::Data<ApiData>,
    id: Option<String>,
    body: LoginBody,
) -> impl Future<Item = LoginResponse, Error = ApiError> {
    let (data1, data2, body1) = (data.clone(), data.clone(), body.clone());

    web::block(move || authenticate(&data, id))
        .map_err(Into::into)
        .and_then(move |service| {
            check_password_pwned(&data1, &body.password)
                .map(|password_pwned| (service, password_pwned))
        })
        .and_then(move |(service, _password_pwned)| {
            web::block(move || {
                data2
                    .db
                    .auth_login(&body1.email, &body1.password, &service)
                    // Map invalid password, not found errors to bad request to prevent leakage.
                    // TODO(feature): Warning logs for bad requests.
                    .map_err(|e| match e {
                        DbError::InvalidPassword | DbError::NotFound => ApiError::BadRequest,
                        _e => ApiError::Db(_e),
                    })
            })
            .map_err(Into::into)
        })
}

pub fn check_password_pwned(
    _data: &web::Data<ApiData>,
    password: &str,
) -> impl Future<Item = bool, Error = ApiError> {
    use sha1::{Digest, Sha1};

    let mut hasher = Sha1::new();
    hasher.input(password);
    let _hash = format!("{:.5X}", hasher.result());

    future::ok(false)

    // TODO(feature): Pwned password check in hash_password.
    // let client = actix_web::client::Client::new();
    // client
    //     .get("https://graph.microsoft.com/v1.0/me")
    //     .header(header::CONTENT_TYPE, ContentType::plaintext())
    //     .header(header::USER_AGENT, data.user_agent())
    //     .send()
    //     .map_err(|_e| ApiError::Unwrap("failed to client.request"))
    //     .and_then(|response| match response.status() {
    //         StatusCode::OK => future::ok(response),
    //         _ => future::err(ApiError::Unwrap("failed to receive ok response")),
    //     })
    //     .and_then(|mut response| {
    //         // response
    //         //     .json::<MicrosoftUser>()
    //         //     .map_err(|_e| ApiError::Unwrap("failed to parse json"))
    //     })
    //     .map(move |response| (data, response.mail, service_id))
}

// const client = this.restifyClients.createStringClient(
//     `https://api.pwnedpasswords.com/range/${sha1Hash.substr(0, 5)}`,
// );
// const response = await client.get("");
// const index: { [key: string]: number } = {};
// if (response.data != null) {
//     response.data.split("\r\n").map((line) => {
//         const [hash, count] = line.split(":");
//         index[hash.trim()] = Number(count.trim());
//     });
// }
// return has(index, sha1Hash.toUpperCase().substring(5));
