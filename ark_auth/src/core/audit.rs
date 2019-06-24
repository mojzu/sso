use crate::core::{Audit, AuditMeta, Error, Key, Service, User};
use crate::driver;
use chrono::Utc;
use serde::ser::Serialize;
use serde_json::Value;
use time::Duration;

/// Audit paths.
pub enum AuditPath {
    // TODO(refactor): Login type/provider.
    Login,
    LoginError(AuditMessage<AuditLoginError>),
}

/// Audit data message container.
#[derive(Debug, Serialize)]
pub struct AuditMessage<T: Serialize> {
    pub message: T,
}

/// Audit message generic trait.
pub trait ToAuditMessage<T: Serialize> {
    /// Convert type to serialisable audit message.
    fn to_audit_message(self) -> AuditMessage<T>;
}

/// Audit login error messages.
#[derive(Debug, Serialize)]
pub enum AuditLoginError {
    UserNotFoundOrDisabled,
    KeyNotFoundOrDisabled,
    PasswordIncorrect,
}

impl ToAuditMessage<AuditLoginError> for AuditLoginError {
    fn to_audit_message(self) -> AuditMessage<AuditLoginError> {
        AuditMessage { message: self }
    }
}

impl AuditPath {
    /// Return string representation and JSON value of key.
    pub fn to_path_data(&self) -> (String, Value) {
        match self {
            AuditPath::Login => ("ark_auth/login".to_owned(), Value::default()),
            AuditPath::LoginError(message) => {
                let value = serde_json::to_value(message).unwrap();
                ("ark_auth/login_error".to_owned(), value)
            }
        }
    }
}

/// Audit log builder pattern.
pub struct AuditBuilder<'a> {
    driver: &'a driver::Driver,
    meta: &'a AuditMeta,
    key: &'a Key,
    service: Option<&'a Service>,
    user: Option<&'a User>,
    user_key: Option<&'a Key>,
}

impl<'a> AuditBuilder<'a> {
    /// Create a new audit log builder with required parameters.
    pub fn new(driver: &'a driver::Driver, meta: &'a AuditMeta, key: &'a Key) -> Self {
        AuditBuilder {
            driver,
            meta,
            key,
            service: None,
            user: None,
            user_key: None,
        }
    }

    pub fn set_service(mut self, service: Option<&'a Service>) -> Self {
        self.service = service;
        self
    }

    pub fn set_user(mut self, user: Option<&'a User>) -> Self {
        self.user = user;
        self
    }

    pub fn set_user_key(mut self, key: Option<&'a Key>) -> Self {
        self.user_key = key;
        self
    }

    /// Create audit log from internal parameters.
    /// In case of error, log as warning and return None.
    pub fn create(&self, path: AuditPath) -> Option<Audit> {
        match create(
            self.driver,
            self.meta,
            path,
            self.key,
            self.service,
            self.user,
            self.user_key,
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
    key: &Key,
    service: Option<&Service>,
    user: Option<&User>,
    user_key: Option<&Key>,
) -> Result<Audit, Error> {
    let (path, data) = path.to_path_data();
    driver
        .audit_create(
            &meta.user_agent,
            &meta.remote,
            meta.forwarded_for.as_ref().map(|x| &**x),
            &path,
            &data,
            &key.id,
            service.map(|x| x.id.as_ref()),
            user.map(|x| x.id.as_ref()),
            user_key.map(|x| x.id.as_ref()),
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
