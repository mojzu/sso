use crate::core::{Audit, AuditMeta, AuditQuery, Error, Key, Service, User, DEFAULT_LIMIT};
use crate::driver;
use chrono::Utc;
use serde::ser::Serialize;
use serde_json::Value;
use time::Duration;

/// Audit paths.
#[derive(Debug, Serialize, Deserialize)]
pub enum AuditPath {
    AuthenticationError,
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
            AuditPath::AuthenticationError => format!("{}/error/authentication", prefix),
            AuditPath::Login => format!("{}/login", prefix),
            AuditPath::LoginError => format!("{}/error/login", prefix),
            AuditPath::ResetPassword => format!("{}/reset_password", prefix),
            AuditPath::ResetPasswordError => format!("{}/error/reset_password", prefix),
            AuditPath::ResetPasswordConfirm => format!("{}/reset_password_confirm", prefix),
            AuditPath::ResetPasswordConfirmError => {
                format!("{}/error/reset_password_confirm", prefix)
            }
            AuditPath::UpdateEmail => format!("{}/update_email", prefix),
            AuditPath::UpdateEmailError => format!("{}/error/update_email", prefix),
            AuditPath::UpdateEmailRevoke => format!("{}/update_email_revoke", prefix),
            AuditPath::UpdateEmailRevokeError => format!("{}/error/update_email_revoke", prefix),
            AuditPath::UpdatePassword => format!("{}/update_password", prefix),
            AuditPath::UpdatePasswordError => format!("{}/error/update_password", prefix),
            AuditPath::UpdatePasswordRevoke => format!("{}/update_password_revoke", prefix),
            AuditPath::UpdatePasswordRevokeError => {
                format!("{}/error/update_password_revoke", prefix)
            }
            AuditPath::Oauth2Login => format!("{}/oauth2_login", prefix),
            AuditPath::Oauth2LoginError => format!("{}/error/oauth2_login", prefix),
            AuditPath::KeyVerifyError => format!("{}/error/key_verify", prefix),
            AuditPath::KeyRevoke => format!("{}/key_revoke", prefix),
            AuditPath::KeyRevokeError => format!("{}/error/key_revoke", prefix),
            AuditPath::TokenVerifyError => format!("{}/error/token_verify", prefix),
            AuditPath::TokenRefresh => format!("{}/token_refresh", prefix),
            AuditPath::TokenRefreshError => format!("{}/error/token_refresh", prefix),
            AuditPath::TokenRevoke => format!("{}/token_revoke", prefix),
            AuditPath::TokenRevokeError => format!("{}/error/token_revoke", prefix),
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

    pub fn set_user_id(&mut self, user: Option<String>) -> &mut Self {
        self.user = user.map(|x| x.to_owned());
        self
    }

    pub fn set_user_key(&mut self, key: Option<&Key>) -> &mut Self {
        self.user_key = key.map(|x| x.id.to_owned());
        self
    }

    pub fn set_user_key_id(&mut self, key: Option<String>) -> &mut Self {
        self.user_key = key.map(|x| x.to_owned());
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

    // TODO(refactor): Handle gt AND lt, created_gte AND created_lte cases.
    match &query.gt {
        Some(gt) => driver
            .audit_list_where_id_gt(gt, limit, service_mask)
            .map_err(Error::Driver),
        None => match &query.lt {
            Some(lt) => driver
                .audit_list_where_id_lt(lt, limit, service_mask)
                .map_err(Error::Driver),
            None => match &query.created_gte {
                Some(created_gte) => driver
                    .audit_list_where_created_gte(
                        created_gte,
                        query.offset_id.as_ref().map(|x| &**x),
                        limit,
                        service_mask,
                    )
                    .map_err(Error::Driver),
                None => match &query.created_lte {
                    Some(created_lte) => driver
                        .audit_list_where_created_lte(
                            created_lte,
                            query.offset_id.as_ref().map(|x| &**x),
                            limit,
                            service_mask,
                        )
                        .map_err(Error::Driver),
                    None => driver
                        .audit_list_where_id_gt("", limit, service_mask)
                        .map_err(Error::Driver),
                },
            },
        },
    }
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
    _service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    id: &str,
) -> Result<Option<Audit>, Error> {
    driver.audit_read_by_id(id).map_err(Error::Driver)
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
