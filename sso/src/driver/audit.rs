use crate::{
    impl_enum_to_from_string, pattern::HeaderAuth, Driver, DriverResult, KeyWithValue, Service,
    User,
};
use chrono::{DateTime, Utc};
use serde::ser::Serialize;
use serde_json::Value;
use std::fmt;
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
    AuthLocalRegister,
    AuthLocalRegisterConfirm,
    AuthLocalRegisterRevoke,
    AuthLocalResetPassword,
    AuthLocalResetPasswordConfirm,
    AuthLocalResetPasswordRevoke,
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
    AuthTokenExchange,
    AuthTotp,
    AuthCsrfCreate,
    AuthCsrfVerify,
}

impl_enum_to_from_string!(AuditType, "sso:");

/// Audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audit {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: Uuid,
    pub user_agent: String,
    pub remote: String,
    pub forwarded: Option<String>,
    pub status_code: Option<u16>,
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
    pub status_code: Option<u16>,
    pub type_: String,
    pub subject: Option<String>,
    pub data: Option<Value>,
    pub key_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub user_key_id: Option<Uuid>,
}

impl AuditCreate {
    pub fn new(meta: AuditMeta, type_: String) -> Self {
        Self {
            meta,
            status_code: None,
            type_,
            subject: None,
            data: None,
            key_id: None,
            service_id: None,
            user_id: None,
            user_key_id: None,
        }
    }

    pub fn status_code(mut self, status_code: Option<u16>) -> Self {
        self.status_code = status_code;
        self
    }

    pub fn subject(mut self, subject: Option<String>) -> Self {
        self.subject = subject;
        self
    }

    pub fn data(mut self, data: Option<Value>) -> Self {
        self.data = data;
        self
    }

    pub fn key_id(mut self, key_id: Option<Uuid>) -> Self {
        self.key_id = key_id;
        self
    }

    pub fn service_id(mut self, service_id: Option<Uuid>) -> Self {
        self.service_id = service_id;
        self
    }

    pub fn user_id(mut self, user_id: Option<Uuid>) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn user_key_id(mut self, user_key_id: Option<Uuid>) -> Self {
        self.user_key_id = user_key_id;
        self
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
pub struct AuditList {
    pub query: AuditListQuery,
    pub filter: AuditListFilter,
}

/// Audit read.
#[derive(Debug)]
pub struct AuditRead {
    pub id: Uuid,
    pub subject: Option<String>,
}

impl AuditRead {
    pub fn new(id: Uuid) -> Self {
        Self { id, subject: None }
    }

    pub fn subject(mut self, subject: Option<String>) -> Self {
        self.subject = subject;
        self
    }
}

/// Audit update.
#[derive(Debug)]
pub struct AuditUpdate {
    pub id: Uuid,
    pub status_code: Option<u16>,
    pub subject: Option<String>,
    pub data: Option<Value>,
}

/// Audit metadata.
///
/// HTTP request information.
#[derive(Debug, Clone)]
pub struct AuditMeta {
    user_agent: String,
    remote: String,
    forwarded: Option<String>,
    user: Option<HeaderAuth>,
}

impl AuditMeta {
    /// Create audit metadata from parameters.
    pub fn new<U, R>(
        user_agent: U,
        remote: R,
        forwarded: Option<String>,
        user: Option<HeaderAuth>,
    ) -> Self
    where
        U: Into<String>,
        R: Into<String>,
    {
        AuditMeta {
            user_agent: user_agent.into(),
            remote: remote.into(),
            forwarded,
            user,
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

    /// User authorisation optional reference.
    pub fn user(&self) -> Option<&HeaderAuth> {
        self.user.as_ref()
    }
}

/// Audit log builder pattern.
#[derive(Debug)]
pub struct AuditBuilder {
    meta: AuditMeta,
    type_: AuditType,
    key: Option<Uuid>,
    service: Option<Uuid>,
    user: Option<Uuid>,
    user_key: Option<Uuid>,
}

impl AuditBuilder {
    /// Create a new audit log builder with required parameters.
    pub fn new(meta: AuditMeta, type_: AuditType) -> Self {
        AuditBuilder {
            meta,
            type_,
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

    /// Get reference to metadata.
    pub fn meta(&self) -> &AuditMeta {
        &self.meta
    }

    /// Create audit log from parameters.
    pub fn create<T>(
        &self,
        driver: &dyn Driver,
        type_: T,
        subject: Option<String>,
        data: Option<Value>,
    ) -> DriverResult<Audit>
    where
        T: Into<String>,
    {
        let data = AuditCreate::new(self.meta.clone(), type_.into())
            .subject(subject)
            .data(data)
            .key_id(self.key)
            .service_id(self.service)
            .user_id(self.user)
            .user_key_id(self.user_key);
        driver.audit_create(&data)
    }

    pub fn create2(&self, driver: &dyn Driver, create: &AuditCreate) -> DriverResult<Audit> {
        let data = AuditCreate::new(self.meta.clone(), create.type_.clone())
            .subject(create.subject.clone())
            .data(create.data.clone())
            .key_id(self.key)
            .service_id(self.service)
            .user_id(create.user_id.clone())
            .user_key_id(create.user_key_id.clone());
        driver.audit_create(&data)
    }

    /// Create audit log with data.
    pub fn create_data<S: Serialize>(
        &self,
        driver: &dyn Driver,
        status_code: u16,
        subject: Option<String>,
        data: Option<S>,
    ) -> DriverResult<Audit> {
        let data = data.map(|x| serde_json::to_value(x).unwrap());
        let audit_data = AuditCreate::new(self.meta.clone(), self.type_.to_string().unwrap())
            .status_code(Some(status_code))
            .subject(subject)
            .data(data)
            .key_id(self.key)
            .service_id(self.service)
            .user_id(self.user)
            .user_key_id(self.user_key);
        driver.audit_create(&audit_data)
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
///
/// Internal structure is:
/// key -> previous value -> current value.
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
    /// Compare 2 versions of a value, if different push a row to diff data.
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

    /// Compare 2 versions of an optional value, if different push a row to diff data.
    pub fn compare_opt<T: PartialEq + fmt::Display>(
        mut self,
        key: &str,
        current: Option<&T>,
        previous: Option<&T>,
    ) -> Self {
        match (current, previous) {
            (Some(current), Some(previous)) => {
                if current != previous {
                    self.data.push((
                        String::from(key),
                        format!("{}", previous),
                        format!("{}", current),
                    ))
                }
            }
            (Some(current), _) => {
                self.data
                    .push((String::from(key), format!(""), format!("{}", current)))
            }
            (_, Some(previous)) => {
                self.data
                    .push((String::from(key), format!("{}", previous), format!("")))
            }
            _ => {}
        };
        self
    }

    /// Compare 2 versions of a vector, if different push a row to diff data.
    pub fn compare_vec<T: PartialEq + fmt::Display>(
        mut self,
        key: &str,
        current: &[T],
        previous: &[T],
    ) -> Self {
        if current != previous {
            let previous: Vec<String> = previous.iter().map(|x| format!("{}", x)).collect();
            let previous: String = previous.join(", ");
            let current: Vec<String> = current.iter().map(|x| format!("{}", x)).collect();
            let current: String = current.join(", ");
            self.data.push((String::from(key), previous, current))
        }
        self
    }

    /// Serialise diff data into Value.
    pub fn into_value(self) -> Value {
        Self::typed_data("diff", self.data)
    }

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
}
