use crate::{ClientError, DriverError};
use serde::ser::{Serialize, Serializer};
use std::fmt;

/// Core error causes.
#[derive(Debug)]
pub enum CoreCause {
    ValidateError(String),
    ServiceNotFound,
    ServiceDisabled,
    ServiceProviderLocalUndefined,
    ServiceInvalidUrl,
    ServiceCannotCreateServiceKey,
    UserNotFound,
    UserExists,
    UserDisabled,
    UserKeyTooManyEnabledToken,
    UserKeyTooManyEnabledTotp,
    KeyNotFound,
    KeyInvalid,
    KeyUndefined,
    KeyDisabledOrRevoked,
    PasswordUndefined,
    PasswordUpdateRequired,
    PasswordNotSetOrIncorrect,
    ResetPasswordDisabled,
    TokenInvalidOrExpired,
    CsrfNotFoundOrUsed,
    UpdateEmailRevoke,
    UpdatePasswordRevoke,
    ServiceMismatch,
    TotpInvalid,
    JwtInvalidClaimsType,
    JwtClaimsTypeMismatch,
    JwtServiceMismatch,
    NotifySendError,
    PwnedPasswordsDisabled,
    GithubOauth2Disabled,
    MicrosoftOauth2Disabled,
    AuditNotFound,
}

impl fmt::Display for CoreCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Core errors.
#[derive(Debug, Fail)]
pub enum CoreError {
    #[fail(display = "CoreError:BadRequest {}", _0)]
    BadRequest(CoreCause),

    #[fail(display = "CoreError:Unauthorised {}", _0)]
    Unauthorised(CoreCause),

    #[fail(display = "CoreError:Forbidden {}", _0)]
    Forbidden(CoreCause),

    #[fail(display = "CoreError:NotFound {}", _0)]
    NotFound(CoreCause),

    #[fail(display = "CoreError:Oauth2Request {}", _0)]
    Oauth2Request(failure::Error),

    #[fail(display = "CoreError:UrlParse {}", _0)]
    UrlParse(#[fail(cause)] url::ParseError),

    #[fail(display = "CoreError:Metrics")]
    Metrics,

    #[fail(display = "CoreError:Driver {}", _0)]
    Driver(#[fail(cause)] DriverError),

    #[fail(display = "CoreError:Client {}", _0)]
    Client(#[fail(cause)] ClientError),

    #[fail(display = "CoreError:LibreauthPass {}", _0)]
    LibreauthPass(usize),

    #[fail(display = "CoreError:LibreauthOath {}", _0)]
    LibreauthOath(usize),

    #[fail(display = "CoreError:Jsonwebtoken {}", _0)]
    Jsonwebtoken(#[fail(cause)] jsonwebtoken::errors::Error),

    #[fail(display = "CoreError:UuidParse {}", _0)]
    UuidParse(#[fail(cause)] uuid::parser::ParseError),

    #[fail(display = "CoreError:ActixMailbox {}", _0)]
    ActixMailbox(#[fail(cause)] actix::MailboxError),

    #[fail(display = "CoreError:SerdeJson {}", _0)]
    SerdeJson(#[fail(cause)] serde_json::Error),

    #[fail(display = "CoreError:SerdeQs {}", _0)]
    SerdeQs(String),

    #[fail(display = "CoreError:Zxcvbn {}", _0)]
    Zxcvbn(#[fail(cause)] zxcvbn::ZxcvbnError),
}

/// Core result wrapper type.
pub type CoreResult<T> = Result<T, CoreError>;

impl CoreError {
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

impl Serialize for CoreError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let v = format!("{}", self);
        serializer.serialize_str(&v)
    }
}
