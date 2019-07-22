use crate::core::{Audit, AuditMeta, AuditQuery, Error, Key, Service, User, DEFAULT_LIMIT};
use crate::driver;
use chrono::Utc;
use serde::ser::Serialize;
use serde_json::Value;
use time::Duration;

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
}

impl AuditPath {
    /// Return string representation and JSON value of key.
    pub fn to_string(&self) -> String {
        let prefix = crate_name!();
        match self {
            AuditPath::AuthenticateError => format!("{}_authenticate_error", prefix),
            AuditPath::Login => format!("{}_login", prefix),
            AuditPath::LoginError => format!("{}_login_error", prefix),
            AuditPath::ResetPassword => format!("{}_reset_password", prefix),
            AuditPath::ResetPasswordError => format!("{}_reset_password_error", prefix),
            AuditPath::ResetPasswordConfirm => format!("{}_reset_password_confirm", prefix),
            AuditPath::ResetPasswordConfirmError => {
                format!("{}_reset_password_confirm_error", prefix)
            }
            AuditPath::UpdateEmail => format!("{}_update_email", prefix),
            AuditPath::UpdateEmailError => format!("{}_update_email_error", prefix),
            AuditPath::UpdateEmailRevoke => format!("{}_update_email_revoke", prefix),
            AuditPath::UpdateEmailRevokeError => format!("{}_update_email_revoke_error", prefix),
            AuditPath::UpdatePassword => format!("{}_update_password", prefix),
            AuditPath::UpdatePasswordError => format!("{}_update_password_error", prefix),
            AuditPath::UpdatePasswordRevoke => format!("{}_update_password_revoke", prefix),
            AuditPath::UpdatePasswordRevokeError => {
                format!("{}_update_password_revoke_error", prefix)
            }
            AuditPath::Oauth2Login => format!("{}_oauth2_login", prefix),
            AuditPath::Oauth2LoginError => format!("{}_oauth2_login_error", prefix),
            AuditPath::KeyVerifyError => format!("{}_key_verify_error", prefix),
            AuditPath::KeyRevoke => format!("{}_key_revoke", prefix),
            AuditPath::KeyRevokeError => format!("{}_key_revoke_error", prefix),
            AuditPath::TokenVerifyError => format!("{}_token_verify_error", prefix),
            AuditPath::TokenRefresh => format!("{}_token_refresh", prefix),
            AuditPath::TokenRefreshError => format!("{}_token_refresh_error", prefix),
            AuditPath::TokenRevoke => format!("{}_token_revoke", prefix),
            AuditPath::TokenRevokeError => format!("{}_token_revoke_error", prefix),
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

    pub fn set_key(&mut self, key: Option<&Key>) -> &mut Self {
        self.key = key.map(|x| x.id.to_owned());
        self
    }

    pub fn set_service(&mut self, service: Option<&Service>) -> &mut Self {
        self.service = service.map(|x| x.id.to_owned());
        self
    }

    pub fn set_user(&mut self, user: Option<&User>) -> &mut Self {
        self.user = user.map(|x| x.id.to_owned());
        self
    }

    pub fn set_user_id<T: Into<Option<String>>>(&mut self, user: T) -> &mut Self {
        self.user = user.into();
        self
    }

    pub fn set_user_key(&mut self, key: Option<&Key>) -> &mut Self {
        self.user_key = key.map(|x| x.id.to_owned());
        self
    }

    pub fn set_user_key_id<T: Into<Option<String>>>(&mut self, key: T) -> &mut Self {
        self.user_key = key.into();
        self
    }

    /// Create audit log from internal parameters.
    pub fn create(
        &self,
        driver: &driver::Driver,
        path: &str,
        data: &Value,
    ) -> Result<Audit, Error> {
        create(
            driver,
            &self.meta,
            path,
            data,
            self.key.as_ref().map(|x| &**x),
            self.service.as_ref().map(|x| &**x),
            self.user.as_ref().map(|x| &**x),
            self.user_key.as_ref().map(|x| &**x),
        )
    }

    /// Create audit log from internal parameters.
    /// In case of error, log as warning and return None.
    pub fn create_internal(
        &self,
        driver: &driver::Driver,
        path: AuditPath,
        data: AuditMessage,
    ) -> Option<Audit> {
        match create_internal(
            driver,
            &self.meta,
            path,
            data,
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

/// List audit IDs.
pub fn list(
    driver: &driver::Driver,
    service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    query: &AuditQuery,
) -> Result<Vec<String>, Error> {
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
    let service_mask = service_mask.map(|s| s.id.as_ref());

    match (&query.gt, &query.lt) {
        (Some(gt), Some(lt)) => driver.audit_list_where_id_gt_and_lt(gt, lt, limit, service_mask),
        (Some(gt), None) => driver.audit_list_where_id_gt(gt, limit, service_mask),
        (None, Some(lt)) => driver.audit_list_where_id_lt(lt, limit, service_mask),
        (None, None) => {
            let offset_id = query.offset_id.as_ref().map(|x| &**x);
            match (&query.created_gte, &query.created_lte) {
                (Some(created_gte), Some(created_lte)) => driver
                    .audit_list_where_created_gte_and_lte(
                        created_gte,
                        created_lte,
                        offset_id,
                        limit,
                        service_mask,
                    ),
                (Some(created_gte), None) => {
                    driver.audit_list_where_created_gte(created_gte, offset_id, limit, service_mask)
                }
                (None, Some(created_lte)) => {
                    driver.audit_list_where_created_lte(created_lte, offset_id, limit, service_mask)
                }
                (None, None) => driver.audit_list_where_id_gt("", limit, service_mask),
            }
        }
    }
    .map_err(Error::Driver)
}

/// Create one audit log.
pub fn create(
    driver: &driver::Driver,
    meta: &AuditMeta,
    path: &str,
    data: &Value,
    key: Option<&str>,
    service: Option<&str>,
    user: Option<&str>,
    user_key: Option<&str>,
) -> Result<Audit, Error> {
    driver
        .audit_create(meta, path, data, key, service, user, user_key)
        .map_err(Error::Driver)
}

/// Create one audit log.
pub fn create_internal(
    driver: &driver::Driver,
    meta: &AuditMeta,
    path: AuditPath,
    data: AuditMessage,
    key: Option<&str>,
    service: Option<&str>,
    user: Option<&str>,
    user_key: Option<&str>,
) -> Result<Audit, Error> {
    let path = path.to_string();
    let data: AuditMessageObject<AuditMessage> = data.into();
    let data = serde_json::to_value(data).unwrap();
    driver
        .audit_create(meta, &path, &data, key, service, user, user_key)
        .map_err(Error::Driver)
}

/// Read audit by ID.
pub fn read_by_id(
    driver: &driver::Driver,
    service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    id: &str,
) -> Result<Option<Audit>, Error> {
    driver
        .audit_read_by_id(id, service_mask.map(|s| s.id.as_ref()))
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
