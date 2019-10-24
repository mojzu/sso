use crate::{ClientError, DriverError};
use serde_json::Value;

/// Core errors.
#[derive(Debug, Fail)]
pub enum CoreError {
    #[fail(display = "AuditNotFound")]
    AuditNotFound,

    #[fail(display = "KeyNotFound")]
    KeyNotFound,

    #[fail(display = "KeyUndefined")]
    KeyUndefined,

    #[fail(display = "KeyServiceUndefined")]
    KeyServiceUndefined,

    #[fail(display = "KeyUserTokenConstraint")]
    KeyUserTokenConstraint,

    #[fail(display = "KeyUserTotpConstraint")]
    KeyUserTotpConstraint,

    #[fail(display = "KeyDisabled")]
    KeyDisabled,

    #[fail(display = "KeyRevoked")]
    KeyRevoked,

    #[fail(display = "ServiceNotFound")]
    ServiceNotFound,

    #[fail(display = "ServiceProviderMicrosoftOauth2Disabled")]
    ServiceProviderMicrosoftOauth2Disabled,

    #[fail(display = "ServiceProviderGithubOauth2Disabled")]
    ServiceProviderGithubOauth2Disabled,

    #[fail(display = "ServiceCannotCreateServiceKey")]
    ServiceCannotCreateServiceKey,

    #[fail(display = "UserNotFound")]
    UserNotFound,

    #[fail(display = "UserDisabled")]
    UserDisabled,

    #[fail(display = "UserPasswordUpdateRequired")]
    UserPasswordUpdateRequired,

    #[fail(display = "UserResetPasswordDisabled")]
    UserResetPasswordDisabled,

    #[fail(display = "JwtClaimsTypeInvalid")]
    JwtClaimsTypeInvalid,

    #[fail(display = "JwtServiceMismatch")]
    JwtServiceMismatch,

    #[fail(display = "JwtClaimsTypeMismatch")]
    JwtClaimsTypeMismatch,

    #[fail(display = "JwtInvalidOrExpired")]
    JwtInvalidOrExpired,

    #[fail(display = "CsrfNotFoundOrUsed")]
    CsrfNotFoundOrUsed,

    #[fail(display = "CsrfServiceMismatch")]
    CsrfServiceMismatch,

    #[fail(display = "TotpInvalid")]
    TotpInvalid,

    #[fail(display = "PwnedPasswordsDisabled")]
    PwnedPasswordsDisabled,

    #[fail(display = "Validate {}", _0)]
    Validate(Value),

    #[fail(display = "NotifySendError")]
    NotifySendError,

    #[fail(display = "Driver {}", _0)]
    Driver(#[fail(cause)] DriverError),

    #[fail(display = "Client {}", _0)]
    Client(#[fail(cause)] ClientError),

    #[fail(display = "Oauth2Request {}", _0)]
    Oauth2Request(failure::Error),

    #[fail(display = "UrlParse {}", _0)]
    UrlParse(#[fail(cause)] url::ParseError),

    #[fail(display = "LibreauthOath {}", _0)]
    LibreauthOath(usize),

    #[fail(display = "Jsonwebtoken {}", _0)]
    Jsonwebtoken(#[fail(cause)] jsonwebtoken::errors::Error),

    #[fail(display = "UuidParse {}", _0)]
    UuidParse(#[fail(cause)] uuid::parser::ParseError),

    #[fail(display = "ActixMailbox {}", _0)]
    ActixMailbox(#[fail(cause)] actix::MailboxError),

    #[fail(display = "Zxcvbn {}", _0)]
    Zxcvbn(#[fail(cause)] zxcvbn::ZxcvbnError),

    #[fail(display = "Metrics")]
    Metrics,

    #[fail(display = "HttpHeader")]
    HttpHeader,

    #[fail(display = "ActixWebBlockingCancelled")]
    ActixWebBlockingCancelled,
}

/// Core result wrapper type.
pub type CoreResult<T> = Result<T, CoreError>;

impl CoreError {
    pub fn validate(e: validator::ValidationErrors) -> Self {
        Self::Validate(serde_json::to_value(e).unwrap())
    }

    pub fn libreauth_oath(e: libreauth::oath::ErrorCode) -> Self {
        Self::LibreauthOath(e as usize)
    }
}

impl From<DriverError> for CoreError {
    fn from(e: DriverError) -> Self {
        Self::Driver(e)
    }
}

impl From<ClientError> for CoreError {
    fn from(e: ClientError) -> Self {
        Self::Client(e)
    }
}
