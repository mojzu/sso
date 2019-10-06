use crate::{ClientError, DriverError};

/// Core OAuth2 errors.
#[derive(Debug, Fail)]
pub enum CoreOauth2Error {
    #[fail(display = "CoreOauth2Error:Disabled")]
    Disabled,

    #[fail(display = "CoreOauth2Error:Csrf")]
    Csrf,

    #[fail(display = "CoreOauth2Error:Oauth2Request {}", _0)]
    Oauth2Request(failure::Error),
}

/// Core errors.
#[derive(Debug, Fail)]
pub enum CoreError {
    #[fail(display = "CoreError:BadRequest")]
    BadRequest,

    #[fail(display = "CoreError:Forbidden")]
    Forbidden,

    #[fail(display = "CoreError:Unauthorised")]
    Unauthorised,

    #[fail(display = "CoreError:NotFound")]
    NotFound,

    #[fail(display = "CoreError:PwnedPasswordsDisabled")]
    PwnedPasswordsDisabled,

    #[fail(display = "CoreError:Oauth2 {}", _0)]
    Oauth2(CoreOauth2Error),

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
