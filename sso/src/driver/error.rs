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

    #[fail(display = "ServiceUserRegisterDisabled")]
    ServiceUserRegisterDisabled,

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

    #[fail(display = "SmtpDisabled")]
    SmtpDisabled,

    #[fail(display = "PwnedPasswordsDisabled")]
    PwnedPasswordsDisabled,

    #[fail(display = "AuthenticateKeyOrTokenUndefined")]
    AuthenticateKeyOrTokenUndefined,

    #[fail(display = "AuthenticateTypeNotFound")]
    AuthenticateTypeNotFound,

    #[fail(display = "AuthenticateTypeUnknown")]
    AuthenticateTypeUnknown,

    #[fail(display = "Locked {:?}", _0)]
    Locked(i32),

    #[fail(display = "Message {}", _0)]
    Message(String),

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

    #[fail(display = "SerdeUrlencoded {}", _0)]
    SerdeUrlencoded(String),

    #[fail(display = "UuidParse {}", _0)]
    UuidParse(#[fail(cause)] uuid::parser::ParseError),

    #[fail(display = "Reqwest {}", _0)]
    Reqwest(#[fail(cause)] reqwest::Error),

    #[fail(display = "UrlParse {}", _0)]
    UrlParse(#[fail(cause)] url::ParseError),

    #[fail(display = "Zxcvbn {}", _0)]
    Zxcvbn(#[fail(cause)] zxcvbn::ZxcvbnError),

    #[fail(display = "Oauth2Request {}", _0)]
    Oauth2Request(failure::Error),

    #[fail(display = "HandlebarsRender {}", _0)]
    HandlebarsRender(#[fail(cause)] handlebars::RenderError),

    #[fail(display = "NativeTls {}", _0)]
    NativeTls(#[fail(cause)] native_tls::Error),

    #[fail(display = "Lettre {}", _0)]
    Lettre(#[fail(cause)] lettre::smtp::error::Error),

    #[fail(display = "LettreEmail {}", _0)]
    LettreEmail(#[fail(cause)] lettre_email::error::Error),

    #[fail(display = "Lettre {}", _0)]
    LettreFile(#[fail(cause)] lettre::file::error::Error),

    #[fail(display = "Metrics")]
    Metrics,

    #[fail(display = "HttpHeader")]
    HttpHeader,

    #[fail(display = "ActixWebBlockingCancelled")]
    ActixWebBlockingCancelled,

    #[fail(display = "Validation {}", _0)]
    Validation(#[fail(cause)] validator::ValidationErrors),
}

impl From<libreauth::pass::ErrorCode> for DriverError {
    fn from(e: libreauth::pass::ErrorCode) -> Self {
        Self::LibreauthPass(e as usize)
    }
}

impl From<libreauth::oath::ErrorCode> for DriverError {
    fn from(e: libreauth::oath::ErrorCode) -> Self {
        Self::LibreauthOath(e as usize)
    }
}

impl From<serde_urlencoded::ser::Error> for DriverError {
    fn from(e: serde_urlencoded::ser::Error) -> Self {
        Self::SerdeUrlencoded(format!("{}", e))
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
