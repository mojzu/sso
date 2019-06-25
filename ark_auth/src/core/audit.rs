use crate::core::{Audit, Error, Key, Service, User};
use crate::driver;
use chrono::Utc;
use serde::ser::Serialize;
use serde_json::Value;
use time::Duration;

/// Audit meta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMeta {
    pub user_agent: String,
    pub remote: String,
    pub forwarded_for: Option<String>,
}

/// Audit paths.
pub enum AuditPath {
    // TODO(refactor): Login type/provider.
    Login(AuditMessageObject<AuditMessage>),
    LoginError(AuditMessageObject<AuditMessage>),
}

impl AuditPath {
    /// Return string representation and JSON value of key.
    pub fn to_path_data(&self) -> (String, Value) {
        match self {
            AuditPath::Login(message) => {
                let value = serde_json::to_value(message).unwrap();
                ("ark_auth/login".to_owned(), value)
            }
            AuditPath::LoginError(message) => {
                let value = serde_json::to_value(message).unwrap();
                ("ark_auth/login_error".to_owned(), value)
            }
        }
    }
}

/// Audit login messages.
#[derive(Debug, Serialize)]
pub enum AuditMessage {
    UserNotFoundOrDisabled,
    KeyNotFoundOrDisabled,
    PasswordIncorrect,
    Login,
}

/// Audit data message container.
#[derive(Debug, Serialize)]
pub struct AuditMessageObject<T: Serialize> {
    pub message: T,
}

impl From<AuditMessage> for AuditMessageObject<AuditMessage> {
    fn from(message: AuditMessage) -> AuditMessageObject<AuditMessage> {
        AuditMessageObject { message }
    }
}

/// Audit log builder pattern.
pub struct AuditBuilder {
    meta: AuditMeta,
    key: Option<String>,
    service: Option<String>,
    user: Option<String>,
    user_key: Option<String>,
}

impl AuditBuilder {
    /// Create a new audit log builder with required parameters.
    pub fn new(meta: AuditMeta) -> Self {
        AuditBuilder {
            meta,
            key: None,
            service: None,
            user: None,
            user_key: None,
        }
    }

    pub fn set_key(mut self, key: Option<&Key>) -> Self {
        self.key = key.map(|x| x.id.to_owned());
        self
    }

    pub fn set_service(mut self, service: Option<&Service>) -> Self {
        self.service = service.map(|x| x.id.to_owned());
        self
    }

    pub fn set_user(mut self, user: Option<&User>) -> Self {
        self.user = user.map(|x| x.id.to_owned());
        self
    }

    pub fn set_user_key(mut self, key: Option<&Key>) -> Self {
        self.user_key = key.map(|x| x.id.to_owned());
        self
    }

    /// Create audit log from internal parameters.
    /// In case of error, log as warning and return None.
    pub fn create(&self, driver: &driver::Driver, path: AuditPath) -> Option<Audit> {
        match create(
            driver,
            &self.meta,
            path,
            self.key.as_ref().map(|x| &**x),
            self.service.as_ref().map(|x| &**x),
            self.user.as_ref().map(|x| &**x),
            self.user_key.as_ref().map(|x| &**x),
        ) {
            Ok(audit) => Some(audit),
            Err(err) => {
                warn!("{}", err);
                None
            }
        }
    }
}

/// Create one audit log.
pub fn create(
    driver: &driver::Driver,
    meta: &AuditMeta,
    path: AuditPath,
    key: Option<&str>,
    service: Option<&str>,
    user: Option<&str>,
    user_key: Option<&str>,
) -> Result<Audit, Error> {
    let (path, data) = path.to_path_data();
    driver
        .audit_create(
            &meta.user_agent,
            &meta.remote,
            meta.forwarded_for.as_ref().map(|x| &**x),
            &path,
            &data,
            key,
            service,
            user,
            user_key,
        )
        .map_err(Error::Driver)
}

/// Delete many audit logs older than days.
pub fn delete_by_age(driver: &driver::Driver, days: usize) -> Result<usize, Error> {
    let days: i64 = 0 - days as i64;
    let created_at = Utc::now() + Duration::days(days);
    match driver.audit_delete_by_created_at(&created_at) {
        Ok(count) => Ok(count),
        Err(err) => {
            warn!("{}", Error::Driver(err));
            Ok(0)
        }
    }
}
