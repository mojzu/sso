use crate::{Core, CoreError, CoreResult, Driver, Key, Service, User};
use chrono::{DateTime, Utc};
use serde::ser::Serialize;
use serde_json::Value;
use std::fmt;
use time::Duration;
use uuid::Uuid;

/// Audit path maximum length.
pub const AUDIT_PATH_MAX_LEN: usize = 200;

/// Audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audit {
    pub created_at: DateTime<Utc>,
    pub id: Uuid,
    pub user_agent: String,
    pub remote: String,
    pub forwarded: Option<String>,
    pub path: String,
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
        write!(f, "\n\tpath {}", self.path)?;
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

/// Audit create data.
pub struct AuditCreate<'a> {
    pub meta: &'a AuditMeta,
    pub path: &'a str,
    pub data: &'a Value,
    pub key_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub user_key_id: Option<Uuid>,
}

impl<'a> AuditCreate<'a> {
    /// New create data reference.
    pub fn new(
        meta: &'a AuditMeta,
        path: &'a str,
        data: &'a Value,
        key_id: Option<Uuid>,
        service_id: Option<Uuid>,
        user_id: Option<Uuid>,
        user_key_id: Option<Uuid>,
    ) -> Self {
        Self {
            meta,
            path,
            data,
            key_id,
            service_id,
            user_id,
            user_key_id,
        }
    }
}

/// Audit list query.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditQuery {
    pub gt: Option<Uuid>,
    pub lt: Option<Uuid>,
    pub created_gte: Option<DateTime<Utc>>,
    pub created_lte: Option<DateTime<Utc>>,
    pub offset_id: Option<Uuid>,
    pub limit: Option<i64>,
}

/// Audit data.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditData {
    pub path: String,
    pub data: Value,
}

/// Audit paths.
#[derive(Debug, Serialize, Deserialize)]
pub enum AuditPath {
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

impl AuditPath {
    /// Return string representation and JSON value of key.
    pub fn to_string(&self) -> String {
        let prefix = crate_name!();
        match self {
            AuditPath::AuthenticateError => format!("{}.error.authenticate", prefix),
            AuditPath::Login => format!("{}.login", prefix),
            AuditPath::LoginError => format!("{}.error.login", prefix),
            AuditPath::ResetPassword => format!("{}.reset_password", prefix),
            AuditPath::ResetPasswordError => format!("{}.error.reset_password", prefix),
            AuditPath::ResetPasswordConfirm => format!("{}.reset_password_confirm", prefix),
            AuditPath::ResetPasswordConfirmError => {
                format!("{}.error.reset_password_confirm", prefix)
            }
            AuditPath::UpdateEmail => format!("{}.update_email", prefix),
            AuditPath::UpdateEmailError => format!("{}.error.update_email", prefix),
            AuditPath::UpdateEmailRevoke => format!("{}.update_email_revoke", prefix),
            AuditPath::UpdateEmailRevokeError => format!("{}.error.update_email_revoke", prefix),
            AuditPath::UpdatePassword => format!("{}.update_password", prefix),
            AuditPath::UpdatePasswordError => format!("{}.error.update_password", prefix),
            AuditPath::UpdatePasswordRevoke => format!("{}.update_password_revoke", prefix),
            AuditPath::UpdatePasswordRevokeError => {
                format!("{}.error.update_password_revoke", prefix)
            }
            AuditPath::Oauth2Login => format!("{}.oauth2_login", prefix),
            AuditPath::Oauth2LoginError => format!("{}.error.oauth2_login", prefix),
            AuditPath::KeyVerifyError => format!("{}.error.key_verify", prefix),
            AuditPath::KeyRevoke => format!("{}.key_revoke", prefix),
            AuditPath::KeyRevokeError => format!("{}.error.key_revoke", prefix),
            AuditPath::TokenVerifyError => format!("{}.error.token_verify", prefix),
            AuditPath::TokenRefresh => format!("{}.token_refresh", prefix),
            AuditPath::TokenRefreshError => format!("{}.error.token_refresh", prefix),
            AuditPath::TokenRevoke => format!("{}.token_revoke", prefix),
            AuditPath::TokenRevokeError => format!("{}.error.token_revoke", prefix),
            AuditPath::TotpError => format!("{}.error.totp", prefix),
        }
    }
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
    PasswordNotSetOrIncorrect,
    Login,
    ResetPassword,
    TokenInvalidOrExpired,
    CsrfNotFoundOrUsed,
    ResetPasswordConfirm,
    UpdateEmail,
    UpdateEmailRevoke,
    UpdatePassword,
    UpdatePasswordRevoke,
    Oauth2Login,
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
    pub fn create(&self, driver: &dyn Driver, path: &str, data: &Value) -> CoreResult<Audit> {
        let data = AuditCreate::new(
            &self.meta,
            path,
            data,
            self.key,
            self.service,
            self.user,
            self.user_key,
        );
        Audit::create(driver, &data)
    }

    /// Create audit log from internal parameters.
    /// In case of error, log as warning and return None.
    pub fn create_unchecked(&self, driver: &dyn Driver, path: &str, data: &Value) -> Option<Audit> {
        match self.create(driver, path, data) {
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
        path: AuditPath,
        data: AuditMessage,
    ) -> Option<Audit> {
        let path = path.to_string();
        let data: AuditMessageObject<AuditMessage> = data.into();
        let data = serde_json::to_value(data).unwrap();
        let audit_data = AuditCreate::new(
            &self.meta,
            &path,
            &data,
            self.key,
            self.service,
            self.user,
            self.user_key,
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
        query: &AuditQuery,
    ) -> CoreResult<Vec<Uuid>> {
        let limit = query.limit.unwrap_or_else(Core::default_limit);
        let service_mask = service_mask.map(|s| s.id);

        match (query.gt, query.lt) {
            (Some(gt), Some(lt)) => {
                driver.audit_list_where_id_gt_and_lt(gt, lt, limit, service_mask)
            }
            (Some(gt), None) => driver.audit_list_where_id_gt(gt, limit, service_mask),
            (None, Some(lt)) => driver.audit_list_where_id_lt(lt, limit, service_mask),
            (None, None) => {
                let offset_id = query.offset_id;
                match (&query.created_gte, &query.created_lte) {
                    (Some(created_gte), Some(created_lte)) => driver
                        .audit_list_where_created_gte_and_lte(
                            created_gte,
                            created_lte,
                            offset_id,
                            limit,
                            service_mask,
                        ),
                    (Some(created_gte), None) => driver.audit_list_where_created_gte(
                        created_gte,
                        offset_id,
                        limit,
                        service_mask,
                    ),
                    (None, Some(created_lte)) => driver.audit_list_where_created_lte(
                        created_lte,
                        offset_id,
                        limit,
                        service_mask,
                    ),
                    (None, None) => driver.audit_list_where_id_gt(Uuid::nil(), limit, service_mask),
                }
            }
        }
        .map_err(CoreError::Driver)
    }

    /// Create one audit log.
    pub fn create(driver: &dyn Driver, data: &AuditCreate) -> CoreResult<Audit> {
        driver.audit_create(data).map_err(CoreError::Driver)
    }

    /// Read audit by ID.
    pub fn read_by_id(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
    ) -> CoreResult<Option<Audit>> {
        driver
            .audit_read_by_id(id, service_mask.map(|s| s.id))
            .map_err(CoreError::Driver)
    }

    /// Delete many audit logs older than days.
    pub fn delete_by_age(driver: &dyn Driver, days: i64) -> CoreResult<usize> {
        let days: i64 = 0 - days;
        let created_at = Utc::now() + Duration::days(days);
        match driver.audit_delete_by_created_at(&created_at) {
            Ok(count) => Ok(count),
            Err(err) => {
                warn!("{}", CoreError::Driver(err));
                Ok(0)
            }
        }
    }
}
