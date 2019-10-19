use crate::CoreError;
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
