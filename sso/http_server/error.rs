use crate::internal::*;
use std::fmt;

/// HTTP Error
#[api_v2_errors(code = 400, code = 401, code = 403, code = 404, code = 500)]
#[derive(Debug)]
pub enum HttpError {
    /// Returned when the request is invalid
    BadRequest(Error),
    /// Returned when the request is not authenticated
    Unauthorized(Error),
    /// Returned when the request is not permitted
    Forbidden(Error),
    /// Returned when the requested resource does not exist
    NotFound(Error),
    /// Returned when server has encountered a situation it doesn't know how to handle
    InternalServerError(Error),
}

impl HttpError {
    /// Bad request error
    pub fn bad_request<E: Into<Error>>(e: E) -> Self {
        Self::BadRequest(e.into())
    }

    /// Unauthorized error
    pub fn unauthorized<E: Into<Error>>(e: E) -> Self {
        Self::Unauthorized(e.into())
    }

    fn error_name(&self) -> String {
        match self {
            Self::BadRequest(_) => "BadRequest",
            Self::Unauthorized(_) => "Unauthorized",
            Self::Forbidden(_) => "Forbidden",
            Self::NotFound(_) => "NotFound",
            Self::InternalServerError(_) => "InternalServerError",
        }
        .to_string()
    }

    fn error_message(&self) -> String {
        match self {
            Self::BadRequest(e) => e.to_string(),
            Self::Unauthorized(e) => e.to_string(),
            Self::Forbidden(e) => e.to_string(),
            Self::NotFound(e) => e.to_string(),
            Self::InternalServerError(e) => e.to_string(),
        }
    }

    fn http_status(&self) -> http::StatusCode {
        match self {
            Self::BadRequest(_) => http::StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => http::StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => http::StatusCode::FORBIDDEN,
            Self::NotFound(_) => http::StatusCode::NOT_FOUND,
            Self::InternalServerError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn serialise(&self) -> String {
        serde_json::to_string(&json!({
            "error": self.error_name(),
            "message": self.error_message()
        }))
        .expect("error serialisation failure")
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_name(), self.error_message())
    }
}

impl actix_http::ResponseError for HttpError {
    fn status_code(&self) -> http::StatusCode {
        self.http_status()
    }

    fn error_response(&self) -> actix_http::Response {
        actix_http::Response::build(self.status_code())
            .content_type("application/json")
            .body(self.serialise())
    }
}

/// HTTP Result
pub type HttpResult<T> = std::result::Result<T, HttpError>;
