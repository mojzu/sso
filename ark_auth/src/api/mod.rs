pub mod auth;
pub mod key;
pub mod service;
pub mod user;

use crate::db::{Db, DbError};
use crate::models::AuthService;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::StatusCode;
use actix_web::middleware::identity::{IdentityPolicy, IdentityService};
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use futures::future;
use serde::de::DeserializeOwned;
use validator::Validate;

// TODO(feature): Audit logging, x-forwarded-for header.
// TODO(feature): Metrics, status/events page.

/// API module errors.
#[derive(Fail, Debug)]
pub enum ApiError {
    #[fail(display = "ApiError {:?}", _0)]
    Unwrap(&'static str),
    /// BadRequest, deserialisation failure.
    #[fail(display = "ApiError::BadRequest")]
    BadRequest,
    /// Forbidden, authentication failure.
    #[fail(display = "ApiError::Forbidden")]
    Forbidden,
    /// Invalid OAuth provider, configuration not available.
    #[fail(display = "ApiError::InvalidOauthProvider")]
    InvalidOauthProvider,
    /// Database module error wrapper.
    #[fail(display = "ApiError::Db {}", _0)]
    Db(#[fail(cause)] DbError),
    /// Blocking error cancelled wrapper.
    #[fail(display = "ApiError::BlockingCancelled")]
    BlockingCancelled,
}

impl From<DbError> for ApiError {
    fn from(e: DbError) -> Self {
        ApiError::Db(e)
    }
}

impl actix_web::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::BadRequest => HttpResponse::BadRequest().finish(),
            ApiError::Forbidden => HttpResponse::Forbidden().finish(),
            ApiError::InvalidOauthProvider => HttpResponse::MethodNotAllowed().finish(),
            ApiError::Db(e) => {
                error!("{}", e);
                HttpResponse::InternalServerError().finish()
            }
            _e => {
                error!("{}", _e);
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

impl From<actix_web::error::BlockingError<ApiError>> for ApiError {
    fn from(e: actix_web::error::BlockingError<ApiError>) -> Self {
        match e {
            actix_web::error::BlockingError::Error(e) => e,
            actix_web::error::BlockingError::Canceled => ApiError::BlockingCancelled,
        }
    }
}

/// API service OAuth provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfigOauthProvider {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
}

/// API service SMTP configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfigSmtp {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

/// API service OAuth configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfigOauth {
    github: Option<ApiConfigOauthProvider>,
    microsoft: Option<ApiConfigOauthProvider>,
}

/// API service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    server_addr: String,
    user_agent: String,
    smtp: Option<ApiConfigSmtp>,
    oauth: ApiConfigOauth,
}

impl ApiConfig {
    /// Construct new configuration.
    pub fn new(server_addr: String) -> Self {
        let user_agent = format!("{}/{}", crate_name!(), crate_version!());
        ApiConfig {
            server_addr,
            user_agent,
            smtp: None,
            oauth: ApiConfigOauth {
                github: None,
                microsoft: None,
            },
        }
    }

    // Set SMTP provider.
    pub fn set_smtp(mut self, host: String, port: u16, user: String, password: String) -> Self {
        self.smtp = Some(ApiConfigSmtp {
            host,
            port,
            user,
            password,
        });
        self
    }

    /// Set GitHub OAuth provider.
    pub fn set_oauth_github(
        mut self,
        client_id: String,
        client_secret: String,
        redirect_url: String,
    ) -> Self {
        self.oauth.github = Some(ApiConfigOauthProvider {
            client_id,
            client_secret,
            redirect_url,
        });
        self
    }

    /// Set Microsoft OAuth provider.
    pub fn set_oauth_microsoft(
        mut self,
        client_id: String,
        client_secret: String,
        redirect_url: String,
    ) -> Self {
        self.oauth.microsoft = Some(ApiConfigOauthProvider {
            client_id,
            client_secret,
            redirect_url,
        });
        self
    }

    pub fn server_addr(&self) -> &str {
        &self.server_addr
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    pub fn smtp(&self) -> Option<&ApiConfigSmtp> {
        self.smtp.as_ref()
    }

    pub fn oauth_github(&self) -> Option<&ApiConfigOauthProvider> {
        self.oauth.github.as_ref()
    }

    pub fn oauth_microsoft(&self) -> Option<&ApiConfigOauthProvider> {
        self.oauth.microsoft.as_ref()
    }
}

/// API service data.
pub struct ApiData {
    config: ApiConfig,
    db: Db,
}

impl ApiData {
    /// Construct new API service data.
    pub fn new(config: ApiConfig, db: Db) -> Self {
        ApiData { config, db }
    }

    /// Configured user agent header value, for outgoing http requests.
    pub fn user_agent(&self) -> &str {
        self.config.user_agent()
    }

    /// Configured SMTP settings.
    pub fn smtp(&self) -> Option<&ApiConfigSmtp> {
        self.config.smtp()
    }

    /// Configured Github OAuth settings.
    pub fn oauth_github(&self) -> Option<&ApiConfigOauthProvider> {
        self.config.oauth_github()
    }

    /// Configured Microsoft OAuth settings.
    pub fn oauth_microsoft(&self) -> Option<&ApiConfigOauthProvider> {
        self.config.oauth_microsoft()
    }
}

pub fn v1_service() -> actix_web::Scope {
    web::scope("/v1")
        .service(web::resource("/ping").route(web::get().to(v1_ping)))
        .service(auth::v1_service())
        .service(user::v1_service())
        .service(service::v1_service())
        .service(key::v1_service())
}

pub fn app(cfg: &mut web::ServiceConfig) {
    cfg.service(v1_service());
}

pub fn start(config: ApiConfig, db: Db) {
    let config_server = config.clone();
    let db_server = db.clone();

    HttpServer::new(move || {
        App::new()
            // Shared data.
            .data(ApiData::new(config_server.clone(), db_server.clone()))
            // Logger middleware.
            .wrap(middleware::Logger::default())
            // TODO(refactor): Sentry middleware support.
            // Authorisation header identity service.
            .wrap(ApiIdentityPolicy::identity_service())
            // API service.
            .configure(app)
            // Default route (method not allowed).
            .default_service(web::route().to(HttpResponse::MethodNotAllowed))
    })
    .bind(config.server_addr())
    .unwrap()
    .start();
}

fn v1_ping() -> actix_web::Result<HttpResponse> {
    let body = r#"pong"#;
    Ok(HttpResponse::build(StatusCode::OK).json(body))
}

/// API identity policy service.
#[derive(Default)]
pub struct ApiIdentityPolicy;

impl ApiIdentityPolicy {
    pub fn identity_service() -> IdentityService<Self> {
        IdentityService::new(ApiIdentityPolicy::default())
    }
}

impl IdentityPolicy for ApiIdentityPolicy {
    type Future = actix_web::Result<Option<String>, actix_web::Error>;
    type ResponseFuture = actix_web::Result<(), actix_web::Error>;

    fn from_request(&self, req: &mut ServiceRequest) -> Self::Future {
        let key = match req.headers().get("Authorization") {
            Some(value) => {
                let value = value.to_str().map_err(|_| ApiError::Forbidden)?;
                Some(value.to_owned())
            }
            None => None,
        };
        Ok(key)
    }

    fn to_response<B>(
        &self,
        _id: Option<String>,
        _changed: bool,
        _res: &mut ServiceResponse<B>,
    ) -> Self::ResponseFuture {
        Ok(())
    }
}

/// Authenticate key provided by identity policy and return associated service.
pub fn authenticate(
    data: &web::Data<ApiData>,
    key_value: Option<String>,
) -> Result<AuthService, ApiError> {
    match key_value {
        Some(key_value) => data
            .db
            .service_read_by_key_value(&key_value)
            .map_err(|e| match e {
                DbError::NotFound => ApiError::Forbidden,
                _ => ApiError::Db(e),
            }),
        None => Err(ApiError::Forbidden),
    }
}

/// Body JSON size limit configuration.
pub fn body_json_config() -> web::JsonConfig {
    web::JsonConfig::default().limit(1024)
}

/// Body from JSON value validation trait.
trait BodyFromValue<T: DeserializeOwned + Validate> {
    /// Extract and validate body from JSON value.
    fn from_value(value: serde_json::Value) -> future::FutureResult<T, ApiError> {
        future::result(
            serde_json::from_value::<T>(value)
                .map_err(|_e| ApiError::BadRequest)
                .and_then(|body| {
                    body.validate().map_err(|_e| ApiError::BadRequest)?;
                    Ok(body)
                }),
        )
    }
}
