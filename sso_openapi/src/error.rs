use hyper::{Body, Response, StatusCode};
use std::error::Error;
use std::fmt;

/// API error.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    status: u16,
    code: usize,
    title: String,
}

impl ApiError {
    /// Returns new `ApiError`.
    pub fn new<T: Into<String>>(status: StatusCode, code: usize, title: T) -> Self {
        Self {
            status: status.into(),
            code,
            title: title.into(),
        }
    }

    /// Returns new `ApiError` with not found status code.
    pub fn not_found<T: Into<String>>(code: usize, title: T) -> Self {
        Self::new(StatusCode::NOT_FOUND, code, title)
    }

    /// Returns new `ApiError` with method not allowed status code.
    pub fn method_not_allowed<T: Into<String>>(code: usize, title: T) -> Self {
        Self::new(StatusCode::METHOD_NOT_ALLOWED, code, title)
    }

    /// Returns `Response` for this error.
    pub fn into_response(self) -> Response<Body> {
        let e = ApiErrors::new(self);
        e.into_response()
    }
}

impl From<serde::de::value::Error> for ApiError {
    fn from(e: serde::de::value::Error) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST.into(),
            code: 1,
            title: e.description().to_owned(),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST.into(),
            code: 1,
            title: e.description().to_owned(),
        }
    }
}

/// API errors vector.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrors {
    errors: Vec<ApiError>,
}

impl ApiErrors {
    /// Returns new `ApiErrors` with contained `ApiError`.
    pub fn new<E>(e: E) -> Self
    where
        E: Into<ApiError>,
    {
        Self {
            errors: vec![e.into()],
        }
    }

    /// Returns `StatusCode` for first error in vector.
    pub fn status(&self) -> StatusCode {
        StatusCode::from_u16(self.errors[0].status).unwrap()
    }

    /// Returns `Response` for these errors.
    pub fn into_response(self) -> Response<Body> {
        let e = serde_json::to_vec(&self).unwrap();
        Response::builder()
            .status(self.status())
            .body(Body::from(e))
            .unwrap()
    }
}

impl fmt::Display for ApiErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} errors", self.errors.len())
    }
}

impl Error for ApiErrors {}
