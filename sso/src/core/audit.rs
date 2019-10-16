use crate::{impl_enum_to_from_string, CoreError, CoreResult, Driver, KeyWithValue, Service, User};
use chrono::{DateTime, Utc};
use serde::ser::Serialize;
use serde_json::Value;
use std::fmt;
use time::Duration;
use uuid::Uuid;

/// Audit type maximum length.
pub const AUDIT_TYPE_MAX_LEN: usize = 200;

/// Audit subject maximum length.
pub const AUDIT_SUBJECT_MAX_LEN: usize = 200;

/// Audit types.
#[derive(Debug, Copy, PartialEq, Clone, Serialize, Deserialize)]
pub enum AuditType {
    Metrics,
    AuditList,
    AuditCreate,
    AuditRead,
    AuditUpdate,
    KeyList,
    KeyCreate,
    KeyRead,
    KeyUpdate,
    KeyDelete,
    ServiceList,
    ServiceCreate,
    ServiceRead,
    ServiceUpdate,
    ServiceDelete,
    UserList,
    UserCreate,
    UserRead,
    UserUpdate,
    UserDelete,
    AuthLocalLogin,
    AuthLocalResetPassword,
    AuthLocalResetPasswordConfirm,
    AuthLocalUpdateEmail,
    AuthLocalUpdateEmailRevoke,
    AuthLocalUpdatePassword,
    AuthLocalUpdatePasswordRevoke,
    AuthGithubOauth2Url,
    AuthGithubOauth2Callback,
    AuthMicrosoftOauth2Url,
    AuthMicrosoftOauth2Callback,
    AuthOauth2Login,
    AuthKeyVerify,
    AuthKeyRevoke,
    AuthTokenVerify,
    AuthTokenRefresh,
    AuthTokenRevoke,
    AuthTotp,
    AuthCsrfCreate,
    AuthCsrfVerify,
}

impl_enum_to_from_string!(AuditType, "Sso");

/// Audit messages.
#[derive(Debug, Serialize, Deserialize)]
pub enum AuditMessage {
    // TODO(refactor): Replace this with CoreError.
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

/// Audit message container.
#[derive(Debug, Serialize)]
struct AuditMessageObject {
    #[serde(rename = "type")]
    type_: AuditType,
    message: AuditMessage,
}

impl AuditMessageObject {
    fn new(type_: AuditType, message: AuditMessage) -> Self {
        Self { type_, message }
    }
}

/// Audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audit {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: Uuid,
    pub user_agent: String,
    pub remote: String,
    pub forwarded: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    pub subject: Option<String>,
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
        if let Some(subject) = &self.subject {
            write!(f, "\n\tsubject {}", subject)?;
        }
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

/// Audit create.
#[derive(Debug)]
pub struct AuditCreate {
    pub meta: AuditMeta,
    pub type_: String,
    pub subject: Option<String>,
    pub data: Option<Value>,
    pub key_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub user_key_id: Option<Uuid>,
}

impl AuditCreate {
    pub fn new(
        meta: AuditMeta,
        type_: String,
        subject: Option<String>,
        data: Option<Value>,
        key_id: Option<Uuid>,
        service_id: Option<Uuid>,
        user_id: Option<Uuid>,
        user_key_id: Option<Uuid>,
    ) -> Self {
        Self {
            meta,
            type_,
            subject,
            data,
            key_id,
            service_id,
            user_id,
            user_key_id,
        }
    }
}

/// Audit create 2.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditCreate2 {
    #[serde(rename = "type")]
    pub type_: String,
    pub subject: Option<String>,
    pub data: Option<Value>,
}

impl AuditCreate2 {
    pub fn new(type_: String, subject: Option<String>, data: Option<Value>) -> Self {
        Self {
            type_,
            subject,
            data,
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
    pub subject: Option<Vec<String>>,
    pub service_id: Option<Vec<Uuid>>,
    pub user_id: Option<Vec<Uuid>>,
}

/// Audit list.
#[derive(Debug)]
pub struct AuditList<'a> {
    pub query: &'a AuditListQuery,
    pub filter: &'a AuditListFilter,
    pub service_id_mask: Option<&'a Uuid>,
}

/// Audit update.
#[derive(Debug)]
pub struct AuditUpdate {
    pub subject: Option<String>,
    pub data: Option<Value>,
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

    pub fn key(&mut self, key: Option<&KeyWithValue>) -> &mut Self {
        self.key = key.map(|x| x.id);
        self
    }

    pub fn service(&mut self, service: Option<&Service>) -> &mut Self {
        self.service = service.map(|x| x.id);
        self
    }

    pub fn user(&mut self, user: Option<&User>) -> &mut Self {
        self.user = user.map(|x| x.id);
        self
    }

    pub fn user_id(&mut self, user: Option<Uuid>) -> &mut Self {
        self.user = user;
        self
    }

    pub fn user_key(&mut self, key: Option<&KeyWithValue>) -> &mut Self {
        self.user_key = key.map(|x| x.id);
        self
    }

    pub fn user_key_id(&mut self, key: Option<Uuid>) -> &mut Self {
        self.user_key = key;
        self
    }

    /// Create audit log from parameters.
    pub fn create(&mut self, driver: &dyn Driver, create: AuditCreate2) -> CoreResult<Audit> {
        let data = AuditCreate::new(
            self.meta.clone(),
            create.type_,
            create.subject,
            create.data,
            self.key,
            self.service,
            self.user,
            self.user_key,
        );
        Audit::create(driver, &data)
    }

    /// Create audit log with data.
    pub fn create_data<S: Serialize>(
        &mut self,
        driver: &dyn Driver,
        type_: AuditType,
        subject: Option<String>,
        data: Option<S>,
    ) -> CoreResult<Audit> {
        let data = data.map(|x| serde_json::to_value(x).unwrap());
        let audit_data = AuditCreate::new(
            self.meta.clone(),
            type_.to_string().unwrap(),
            subject,
            data,
            self.key,
            self.service,
            self.user,
            self.user_key,
        );
        Audit::create(driver, &audit_data)
    }

    /// Create audit log with message.
    pub fn create_message(
        &mut self,
        driver: &dyn Driver,
        type_: AuditType,
        data: AuditMessage,
    ) -> CoreResult<Audit> {
        let data = serde_json::to_value(AuditMessageObject::new(type_, data)).unwrap();
        let audit_data = AuditCreate::new(
            self.meta.clone(),
            type_.to_string().unwrap(),
            None,
            Some(data),
            self.key,
            self.service,
            self.user,
            self.user_key,
        );
        Audit::create(driver, &audit_data)
    }
}

/// Audit subject trait.
pub trait AuditSubject {
    /// Return subject value for audit log.
    fn subject(&self) -> String;
}

/// Audit diff trait.
pub trait AuditDiff {
    /// Return diff object for audit log.
    fn diff(&self, other: &Self) -> Value;
}

/// Audit diff builder pattern.
/// Key -> previous value -> current value.
#[derive(Debug)]
pub struct AuditDiffBuilder {
    data: Vec<(String, String, String)>,
}

impl Default for AuditDiffBuilder {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl AuditDiffBuilder {
    pub fn compare<T: PartialEq + fmt::Display>(
        mut self,
        key: &str,
        current: &T,
        previous: &T,
    ) -> Self {
        if current != previous {
            self.data.push((
                String::from(key),
                format!("{}", previous),
                format!("{}", current),
            ))
        }
        self
    }

    pub fn into_value(self) -> Value {
        Audit::typed_data("diff", self.data)
    }
}

impl Audit {
    /// Wrap serialisable data in object with type property.
    pub fn typed_data<T: Into<String>, D1: Serialize>(type_: T, data: D1) -> Value {
        #[derive(Serialize)]
        struct TypedData<D2: Serialize> {
            #[serde(rename = "type")]
            type_: String,
            data: D2,
        }
        let v = TypedData {
            type_: type_.into(),
            data,
        };
        serde_json::to_value(v).unwrap()
    }

    /// List audit logs.
    pub fn list(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
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

    /// Create audit log.
    pub fn create(driver: &dyn Driver, data: &AuditCreate) -> CoreResult<Audit> {
        driver.audit_create(data).map_err(CoreError::Driver)
    }

    /// Read audit log.
    pub fn read(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        id: Uuid,
    ) -> CoreResult<Option<Audit>> {
        driver
            .audit_read_opt(&id, service_mask.map(|s| &s.id))
            .map_err(CoreError::Driver)
    }

    // Update audit log.
    pub fn update(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        id: &Uuid,
        update: &AuditUpdate,
    ) -> CoreResult<Audit> {
        driver
            .audit_update(id, update, service_mask.map(|s| &s.id))
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
