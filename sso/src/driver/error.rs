use crate::ClientError;
use serde_json::Value;

/// Driver errors.
#[derive(Debug, Fail)]
pub enum DriverError {
    #[fail(display = "AuditNotFound")]
    AuditNotFound,

    #[fail(display = "KeyNotFound")]
    KeyNotFound,

    #[fail(display = "KeyUndefined")]
    KeyUndefined,

    #[fail(display = "KeyServiceUndefined")]
    KeyServiceUndefined,

    #[fail(display = "KeyDisabled")]
    KeyDisabled,

    #[fail(display = "KeyRevoked")]
    KeyRevoked,

    #[fail(display = "KeyUserTokenConstraint")]
    KeyUserTokenConstraint,

    #[fail(display = "KeyUserTotpConstraint")]
    KeyUserTotpConstraint,

    #[fail(display = "ServiceNotFound")]
    ServiceNotFound,

    #[fail(display = "ServiceDisabled")]
    ServiceDisabled,

    #[fail(display = "ServiceProviderLocalDisabled")]
    ServiceProviderLocalDisabled,

    #[fail(display = "ServiceProviderMicrosoftOauth2Disabled")]
    ServiceProviderMicrosoftOauth2Disabled,

    #[fail(display = "ServiceProviderGithubOauth2Disabled")]
    ServiceProviderGithubOauth2Disabled,

    #[fail(display = "ServiceCannotCreateServiceKey")]
    ServiceCannotCreateServiceKey,

    #[fail(display = "UserPasswordUpdateRequired")]
    UserPasswordUpdateRequired,

    #[fail(display = "UserResetPasswordDisabled")]
    UserResetPasswordDisabled,

    #[fail(display = "UserNotFound")]
    UserNotFound,

    #[fail(display = "UserDisabled")]
    UserDisabled,

    #[fail(display = "UserEmailConstraint")]
    UserEmailConstraint,

    #[fail(display = "UserPasswordIncorrect")]
    UserPasswordIncorrect,

    #[fail(display = "UserPasswordUndefined")]
    UserPasswordUndefined,

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

    #[fail(display = "NotifySendError")]
    NotifySendError,

    #[fail(display = "PwnedPasswordsDisabled")]
    PwnedPasswordsDisabled,

    #[fail(display = "Validate {}", _0)]
    Validate(Value),

    #[fail(display = "Locked {:?}", _0)]
    Locked(i32),

    #[fail(display = "LockFn {}", _0)]
    LockFn(String),

    #[fail(display = "Client {}", _0)]
    Client(#[fail(cause)] ClientError),

    #[fail(display = "DieselResult {}", _0)]
    DieselResult(#[fail(cause)] diesel::result::Error),

    #[fail(display = "DieselMigrations {}", _0)]
    DieselMigrations(#[fail(cause)] diesel_migrations::RunMigrationsError),

    #[fail(display = "LibreauthPass {}", _0)]
    LibreauthPass(usize),

    #[fail(display = "LibreauthOath {}", _0)]
    LibreauthOath(usize),

    #[fail(display = "R2d2 {}", _0)]
    R2d2(#[fail(cause)] r2d2::Error),

    #[fail(display = "ActixMailbox {}", _0)]
    ActixMailbox(#[fail(cause)] actix::MailboxError),

    #[fail(display = "Rustls")]
    Rustls,

    #[fail(display = "Jsonwebtoken {}", _0)]
    Jsonwebtoken(#[fail(cause)] jsonwebtoken::errors::Error),

    #[fail(display = "StdIo {}", _0)]
    StdIo(#[fail(cause)] std::io::Error),

    #[fail(display = "EnvParse {}", _0)]
    EnvParse(String),

    #[fail(display = "StdEnvVar {}", _0)]
    StdEnvVar(#[fail(cause)] std::env::VarError),

    #[fail(display = "Prometheus {}", _0)]
    Prometheus(#[fail(cause)] prometheus::Error),

    #[fail(display = "SerdeJson {}", _0)]
    SerdeJson(#[fail(cause)] serde_json::Error),

    #[fail(display = "SerdeQs {}", _0)]
    SerdeQs(String),

    #[fail(display = "UuidParse {}", _0)]
    UuidParse(#[fail(cause)] uuid::parser::ParseError),

    #[fail(display = "UrlParse {}", _0)]
    UrlParse(#[fail(cause)] url::ParseError),

    #[fail(display = "Zxcvbn {}", _0)]
    Zxcvbn(#[fail(cause)] zxcvbn::ZxcvbnError),

    #[fail(display = "Oauth2Request {}", _0)]
    Oauth2Request(failure::Error),

    #[fail(display = "Metrics")]
    Metrics,

    #[fail(display = "HttpHeader")]
    HttpHeader,

    #[fail(display = "ActixWebBlockingCancelled")]
    ActixWebBlockingCancelled,
}

impl DriverError {
    pub fn validate(e: validator::ValidationErrors) -> Self {
        Self::Validate(serde_json::to_value(e).unwrap())
    }

    pub fn libreauth_pass(e: libreauth::pass::ErrorCode) -> Self {
        Self::LibreauthPass(e as usize)
    }

    pub fn libreauth_oath(e: libreauth::oath::ErrorCode) -> Self {
        Self::LibreauthOath(e as usize)
    }

    pub fn serde_qs(e: serde_qs::Error) -> Self {
        Self::SerdeQs(e.description().to_owned())
    }
}

impl From<ClientError> for DriverError {
    fn from(e: ClientError) -> Self {
        Self::Client(e)
    }
}

impl From<diesel::result::Error> for DriverError {
    fn from(e: diesel::result::Error) -> Self {
        Self::DieselResult(e)
    }
}

impl From<diesel_migrations::RunMigrationsError> for DriverError {
    fn from(e: diesel_migrations::RunMigrationsError) -> Self {
        Self::DieselMigrations(e)
    }
}

impl From<r2d2::Error> for DriverError {
    fn from(e: r2d2::Error) -> Self {
        Self::R2d2(e)
    }
}

/// Driver result wrapper type.
pub type DriverResult<T> = Result<T, DriverError>;
