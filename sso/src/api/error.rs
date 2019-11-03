use crate::DriverError;
use actix_web::{error::BlockingError, HttpResponse, ResponseError};
use http::StatusCode;
use serde::ser::{Serialize, Serializer};

/// API errors.
#[derive(Debug, Fail)]
pub enum ApiError {
    #[fail(display = "BadRequest {}", _0)]
    BadRequest(#[fail(cause)] DriverError),

    #[fail(display = "Unauthorised {}", _0)]
    Unauthorised(#[fail(cause)] DriverError),

    #[fail(display = "Forbidden {}", _0)]
    Forbidden(#[fail(cause)] DriverError),

    #[fail(display = "NotFound {}", _0)]
    NotFound(#[fail(cause)] DriverError),

    #[fail(display = "InternalServerError {}", _0)]
    InternalServerError(#[fail(cause)] DriverError),
}

impl ApiError {
    pub fn status_code(&self) -> u16 {
        match self {
            ApiError::BadRequest(_e) => StatusCode::BAD_REQUEST.as_u16(),
            ApiError::Unauthorised(_e) => StatusCode::UNAUTHORIZED.as_u16(),
            ApiError::Forbidden(_e) => StatusCode::FORBIDDEN.as_u16(),
            ApiError::NotFound(_e) => StatusCode::NOT_FOUND.as_u16(),
            ApiError::InternalServerError(_e) => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        }
    }
}

impl Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let v = format!("{}", self);
        serializer.serialize_str(&v)
    }
}

impl From<BlockingError<ApiError>> for ApiError {
    fn from(e: BlockingError<ApiError>) -> Self {
        match e {
            BlockingError::Error(e) => e,
            BlockingError::Canceled => {
                Self::InternalServerError(DriverError::ActixWebBlockingCancelled)
            }
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::BadRequest(_e) => HttpResponse::BadRequest().finish(),
            Self::Unauthorised(_e) => HttpResponse::Unauthorized().finish(),
            Self::Forbidden(_e) => HttpResponse::Forbidden().finish(),
            Self::NotFound(_e) => HttpResponse::NotFound().finish(),
            _ => {
                error!("{}", self);
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        if let Some(status) = e.status() {
            match status {
                StatusCode::BAD_REQUEST => Self::BadRequest(DriverError::Reqwest(e)),
                StatusCode::UNAUTHORIZED => Self::Unauthorised(DriverError::Reqwest(e)),
                StatusCode::FORBIDDEN => Self::Forbidden(DriverError::Reqwest(e)),
                StatusCode::NOT_FOUND => Self::NotFound(DriverError::Reqwest(e)),
                _ => Self::InternalServerError(DriverError::Reqwest(e)),
            }
        } else {
            Self::InternalServerError(DriverError::Reqwest(e))
        }
    }
}

/// API result wrapper type.
pub type ApiResult<T> = Result<T, ApiError>;
