use crate::CoreError;
use actix_web::{error::BlockingError, HttpResponse, ResponseError};
use serde::ser::{Serialize, Serializer};

/// API errors.
#[derive(Debug, Fail)]
pub enum ApiError {
    #[fail(display = "BadRequest {}", _0)]
    BadRequest(#[fail(cause)] CoreError),

    #[fail(display = "Unauthorised {}", _0)]
    Unauthorised(#[fail(cause)] CoreError),

    #[fail(display = "Forbidden {}", _0)]
    Forbidden(#[fail(cause)] CoreError),

    #[fail(display = "NotFound {}", _0)]
    NotFound(#[fail(cause)] CoreError),

    #[fail(display = "InternalServerError {}", _0)]
    InternalServerError(#[fail(cause)] CoreError),
}

/// API result wrapper type.
pub type ApiResult<T> = Result<T, ApiError>;

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
                Self::InternalServerError(CoreError::ActixWebBlockingCancelled)
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
