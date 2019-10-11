use crate::{impl_enum_to_from_string, CoreError, CoreResult, Driver, Key, Service, User};
use chrono::{DateTime, Utc};
use serde::ser::Serialize;
use serde_json::Value;
use std::fmt;
use time::Duration;
use uuid::Uuid;

/// Audit type maximum length.
pub const AUDIT_TYPE_MAX_LEN: usize = 200;

/// Audit types.
#[derive(Debug, Copy, PartialEq, Clone, Serialize, Deserialize)]
pub enum AuditType {
    AuthenticateError,
    Login,
    LoginError,
    ResetPassword,
    ResetPasswordError,
    ResetPasswordConfirm,
    ResetPasswordConfirmError,
    UpdateEmail,
    UpdateEmailError,
    UpdateEmailRevoke,
    UpdateEmailRevokeError,
    UpdatePassword,
    UpdatePasswordError,
    UpdatePasswordRevoke,
    UpdatePasswordRevokeError,
    Oauth2Login,
    Oauth2LoginError,
    KeyVerifyError,
    KeyRevoke,
    KeyRevokeError,
    TokenVerifyError,
    TokenRefresh,
    TokenRefreshError,
    TokenRevoke,
    TokenRevokeError,
    TotpError,
}

impl_enum_to_from_string!(AuditType);

/// Audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audit {
    pub created_at: DateTime<Utc>,
    pub id: Uuid,
    pub user_agent: String,
    pub remote: String,
    pub forwarded: Option<String>,
    pub type_: String,
    pub data: Value,
    pub key_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub user_key_id: Option<Uuid>,
}

impl fmt::Display for Audit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Audit {}", self.id)?;
        write!(f, "\n\tcreated_at {}", self.created_at)?;
        write!(f, "\n\tuser_agent {}", self.user_agent)?;
        write!(f, "\n\tremote {}", self.remote)?;
        if let Some(forwarded) = &self.forwarded {
            write!(f, "\n\tforwarded {}", forwarded)?;
        }
        write!(f, "\n\ttype {}", self.type_)?;
        write!(f, "\n\tdata {}", self.data)?;
        if let Some(key_id) = &self.key_id {
            write!(f, "\n\tkey_id {}", key_id)?;
        }
        if let Some(service_id) = &self.service_id {
            write!(f, "\n\tservice_id {}", service_id)?;
        }
        if let Some(user_id) = &self.user_id {
            write!(f, "\n\tuser_id {}", user_id)?;
        }
        if let Some(user_key_id) = &self.user_key_id {
            write!(f, "\n\tuser_key_id {}", user_key_id)?;
        }
        Ok(())
    }
}

/// Audit create data.
#[derive(Debug)]
pub struct AuditCreate<'a> {
    pub meta: &'a AuditMeta,
    pub type_: &'a str,
    pub data: &'a Value,
    pub key_id: Option<&'a Uuid>,
    pub service_id: Option<&'a Uuid>,
    pub user_id: Option<&'a Uuid>,
    pub user_key_id: Option<&'a Uuid>,
}

impl<'a> AuditCreate<'a> {
    /// New create data reference.
    pub fn new(
        meta: &'a AuditMeta,
        type_: &'a str,
        data: &'a Value,
        key_id: Option<&'a Uuid>,
        service_id: Option<&'a Uuid>,
        user_id: Option<&'a Uuid>,
        user_key_id: Option<&'a Uuid>,
    ) -> Self {
        Self {
            meta,
            type_,
            data,
            key_id,
            service_id,
            user_id,
            user_key_id,
        }
    }
}

/// Audit list query.
#[derive(Debug)]
pub enum AuditListQuery {
    /// Where created less than or equal.
    CreatedLe(DateTime<Utc>, i64, Option<Uuid>),
    /// Where created greater than or equal.
    CreatedGe(DateTime<Utc>, i64, Option<Uuid>),
    /// Where created less than or equal and greater than or equal.
    CreatedLeAndGe(DateTime<Utc>, DateTime<Utc>, i64, Option<Uuid>),
}

/// Audit list filter.
#[derive(Debug)]
pub struct AuditListFilter {
    pub id: Option<Vec<Uuid>>,
    pub type_: Option<Vec<String>>,
    pub service_id: Option<Vec<Uuid>>,
    pub user_id: Option<Vec<Uuid>>,
    // TODO(feature): Data matches filter.
}

/// Audit list.
#[derive(Debug)]
pub struct AuditList<'a> {
    pub query: &'a AuditListQuery,
    pub filter: &'a AuditListFilter,
    pub service_id_mask: Option<&'a Uuid>,
}

/// Audit metadata, HTTP request information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMeta {
    user_agent: String,
    remote: String,
    forwarded: Option<String>,
}

impl AuditMeta {
    /// Create audit metadata from parameters.
    pub fn new<T1: Into<String>, T2: Into<Option<String>>>(
        user_agent: T1,
        remote: T1,
        forwarded: T2,
    ) -> Self {
        AuditMeta {
            user_agent: user_agent.into(),
            remote: remote.into(),
            forwarded: forwarded.into(),
        }
    }

    /// User agent string reference.
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Remote IP string reference.
    pub fn remote(&self) -> &str {
        &self.remote
    }

    /// Forwarded for header optional string reference.
    pub fn forwarded(&self) -> Option<&str> {
        self.forwarded.as_ref().map(|x| &**x)
    }
}

/// Audit data.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditData {
    pub type_: String,
    pub data: Value,
}

/// Audit messages.
#[derive(Debug, Serialize, Deserialize)]
pub enum AuditMessage {
    ServiceNotFound,
    ServiceDisabled,
    UserNotFound,
    UserDisabled,
    KeyNotFound,
    KeyInvalid,
    KeyUndefined,
    KeyDisabledOrRevoked,
    PasswordUpdateRequired,
    PasswordNotSetOrIncorrect,
    Login,
    ResetPassword,
    ResetPasswordDisabled,
    TokenInvalidOrExpired,
    CsrfNotFoundOrUsed,
    ResetPasswordConfirm,
    UpdateEmail,
    UpdateEmailRevoke,
    UpdatePassword,
    UpdatePasswordRevoke,
    Oauth2Login,
    ServiceMismatch,
    KeyRevoke,
    TokenRefresh,
    TokenRevoke,
    TotpInvalid,
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
#[derive(Debug)]
pub struct AuditBuilder {
    meta: AuditMeta,
    key: Option<Uuid>,
    service: Option<Uuid>,
    user: Option<Uuid>,
    user_key: Option<Uuid>,
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

    pub fn set_key(&mut self, key: Option<&Key>) -> &mut Self {
        self.key = key.map(|x| x.id);
        self
    }

    pub fn set_service(&mut self, service: Option<&Service>) -> &mut Self {
        self.service = service.map(|x| x.id);
        self
    }

    pub fn set_user(&mut self, user: Option<&User>) -> &mut Self {
        self.user = user.map(|x| x.id);
        self
    }

    pub fn set_user_id(&mut self, user: Option<Uuid>) -> &mut Self {
        self.user = user;
        self
    }

    pub fn set_user_key(&mut self, key: Option<&Key>) -> &mut Self {
        self.user_key = key.map(|x| x.id);
        self
    }

    pub fn set_user_key_id(&mut self, key: Option<Uuid>) -> &mut Self {
        self.user_key = key;
        self
    }

    /// Create audit log from internal parameters.
    pub fn create(&self, driver: &dyn Driver, type_: &str, data: &Value) -> CoreResult<Audit> {
        let data = AuditCreate::new(
            &self.meta,
            type_,
            data,
            self.key.as_ref(),
            self.service.as_ref(),
            self.user.as_ref(),
            self.user_key.as_ref(),
        );
        Audit::create(driver, &data)
    }

    /// Create audit log from internal parameters.
    /// In case of error, log as warning and return None.
    pub fn create_unchecked(
        &self,
        driver: &dyn Driver,
        type_: &str,
        data: &Value,
    ) -> Option<Audit> {
        match self.create(driver, type_, data) {
            Ok(audit) => Some(audit),
            Err(err) => {
                warn!("{}", err);
                None
            }
        }
    }

    /// Create audit log from internal parameters.
    /// In case of error, log as warning and return None.
    pub fn create_internal(
        &self,
        driver: &dyn Driver,
        type_: AuditType,
        data: AuditMessage,
    ) -> Option<Audit> {
        let type_ = type_.to_string().unwrap();
        let data: AuditMessageObject<AuditMessage> = data.into();
        let data = serde_json::to_value(data).unwrap();
        let audit_data = AuditCreate::new(
            &self.meta,
            &type_,
            &data,
            self.key.as_ref(),
            self.service.as_ref(),
            self.user.as_ref(),
            self.user_key.as_ref(),
        );

        match Audit::create(driver, &audit_data) {
            Ok(audit) => Some(audit),
            Err(err) => {
                warn!("{}", err);
                None
            }
        }
    }
}

impl Audit {
    /// List audit IDs.
    pub fn list(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        query: &AuditListQuery,
        filter: &AuditListFilter,
    ) -> CoreResult<Vec<Audit>> {
        let list = AuditList {
            query,
            filter,
            service_id_mask: service_mask.map(|s| &s.id),
        };
        driver.audit_list(&list).map_err(Into::into)
    }

    /// Create one audit log.
    pub fn create(driver: &dyn Driver, data: &AuditCreate) -> CoreResult<Audit> {
        driver.audit_create(data).map_err(CoreError::Driver)
    }

    /// Read audit by ID.
    pub fn read(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
    ) -> CoreResult<Option<Audit>> {
        driver
            .audit_read_opt(&id, service_mask.map(|s| &s.id))
            .map_err(CoreError::Driver)
    }

    /// Delete many audit logs older than days.
    pub fn delete_many(driver: &dyn Driver, days: i64) -> CoreResult<usize> {
        let days: i64 = 0 - days;
        let created_at = Utc::now() + Duration::days(days);
        match driver.audit_delete(&created_at) {
            Ok(count) => Ok(count),
            Err(err) => {
                warn!("{}", CoreError::Driver(err));
                Ok(0)
            }
        }
    }
}
