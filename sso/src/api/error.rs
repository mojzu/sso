use crate::DriverError;
use tonic::Status;

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

impl From<ApiError> for Status {
    fn from(e: ApiError) -> Self {
        match e {
            ApiError::BadRequest(e) => Status::invalid_argument(format!("{}", e)),
            ApiError::Unauthorised(e) => Status::unauthenticated(format!("{}", e)),
            ApiError::Forbidden(e) => Status::permission_denied(format!("{}", e)),
            ApiError::NotFound(e) => Status::not_found(format!("{}", e)),
            ApiError::InternalServerError(e) => Status::internal(format!("{}", e)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusData {
    code: u16,
    message: String,
}

impl From<&Status> for StatusData {
    fn from(s: &Status) -> Self {
        Self {
            code: s.code() as u16,
            message: s.message().to_owned(),
        }
    }
}

impl StatusData {
    pub fn from_status(status: &Status) -> Self {
        status.into()
    }
}

/// API result wrapper type.
pub type ApiResult<T> = Result<T, Status>;
