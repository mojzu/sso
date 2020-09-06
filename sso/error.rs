use crate::oauth2;

/// Error
///
/// Howto: Add an error variant.
#[derive(Debug)]
pub enum Error {
    Message(String),
    Oauth2(oauth2::ErrorResponse),
    IoError(std::io::Error),
    ConfigError(config::ConfigError),
    TokioPostgresError(tokio_postgres::Error),
    DeadpoolPostgresConfigError(deadpool_postgres::config::ConfigError),
    DeadpoolPoolError(deadpool_postgres::PoolError),
    ReqwestError(reqwest::Error),
    HandlebarsTemplateRender(handlebars::TemplateRenderError),
    Lettre(lettre::smtp::error::Error),
    LettreEmail(lettre_email::error::Error),
    Validation(validator::ValidationErrors),
}

/// Result
pub type Result<T> = std::result::Result<T, Error>;

impl std::string::ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Self::Message(e) => e.to_string(),
            Self::Oauth2(e) => e.to_string(),
            Self::IoError(e) => e.to_string(),
            Self::ConfigError(e) => e.to_string(),
            Self::TokioPostgresError(e) => e.to_string(),
            Self::DeadpoolPostgresConfigError(e) => e.to_string(),
            Self::DeadpoolPoolError(e) => e.to_string(),
            Self::ReqwestError(e) => e.to_string(),
            Self::HandlebarsTemplateRender(e) => e.to_string(),
            Self::Lettre(e) => e.to_string(),
            Self::LettreEmail(e) => e.to_string(),
            Self::Validation(e) => e.to_string(),
        }
    }
}

impl From<Error> for String {
    fn from(e: Error) -> Self {
        e.to_string()
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Self::Message(e.to_string())
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Self::Message(e)
    }
}

impl From<oauth2::ErrorResponse> for Error {
    fn from(e: oauth2::ErrorResponse) -> Self {
        Self::Oauth2(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<config::ConfigError> for Error {
    fn from(e: config::ConfigError) -> Self {
        Self::ConfigError(e)
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(e: tokio_postgres::Error) -> Self {
        Self::TokioPostgresError(e)
    }
}

impl From<deadpool_postgres::config::ConfigError> for Error {
    fn from(e: deadpool_postgres::config::ConfigError) -> Self {
        Self::DeadpoolPostgresConfigError(e)
    }
}

impl From<deadpool_postgres::PoolError> for Error {
    fn from(e: deadpool_postgres::PoolError) -> Self {
        Self::DeadpoolPoolError(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::ReqwestError(e)
    }
}

impl From<handlebars::TemplateRenderError> for Error {
    fn from(e: handlebars::TemplateRenderError) -> Self {
        Self::HandlebarsTemplateRender(e)
    }
}

impl From<lettre::smtp::error::Error> for Error {
    fn from(e: lettre::smtp::error::Error) -> Self {
        Self::Lettre(e)
    }
}

impl From<lettre_email::error::Error> for Error {
    fn from(e: lettre_email::error::Error) -> Self {
        Self::LettreEmail(e)
    }
}

impl From<validator::ValidationErrors> for Error {
    fn from(e: validator::ValidationErrors) -> Self {
        Self::Validation(e)
    }
}
