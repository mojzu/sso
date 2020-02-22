use crate::{
    grpc::{pb, util},
    schema::sso_csrf,
    DriverError, DriverResult,
};
use chrono::{DateTime, Duration, Utc};
use diesel::{prelude::*, PgConnection};
use libreauth::key::KeyBuilder;
use std::fmt;
use uuid::Uuid;

/// CSRF key size in bytes.
pub const CSRF_KEY_BYTES: usize = 11;

/// CSRF key and value.
#[derive(Debug, Clone, Identifiable, Queryable)]
#[table_name = "sso_csrf"]
#[primary_key(key)]
pub struct Csrf {
    created_at: DateTime<Utc>,
    key: String,
    value: String,
    ttl: DateTime<Utc>,
    service_id: Uuid,
}

impl Csrf {
    /// Returns reference to key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns reference to value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns service ID.
    pub fn service_id(&self) -> Uuid {
        self.service_id
    }

    /// Create a CSRF key.
    pub fn create(conn: &PgConnection, create: &CsrfCreate) -> DriverResult<Self> {
        diesel::insert_into(sso_csrf::table)
            .values(create)
            .get_result::<Self>(conn)
            .map_err(Into::into)
    }

    /// Verify a CSRF key for service ID, fails if service did not create key.
    pub fn verify(
        conn: &PgConnection,
        service_id: Uuid,
        csrf_key: Option<String>,
    ) -> DriverResult<Self> {
        let csrf_key = csrf_key.ok_or_else(|| DriverError::CsrfNotFoundOrUsed)?;
        Self::read(conn, &csrf_key)?
            .ok_or_else(|| DriverError::CsrfNotFoundOrUsed)
            .and_then(|csrf| {
                csrf.check_service(service_id)?;
                Ok(csrf)
            })
    }

    /// Read CSRF token. CSRF token is deleted after one read.
    pub fn read<T: AsRef<str>>(conn: &PgConnection, key: T) -> DriverResult<Option<Self>> {
        Self::delete_by_ttl(conn)?;

        let csrf = sso_csrf::table
            .filter(sso_csrf::dsl::key.eq(key.as_ref()))
            .get_result::<Csrf>(conn)
            .optional()
            .map_err(DriverError::DieselResult)?;

        if csrf.is_some() {
            Self::delete_by_key(conn, key.as_ref())?;
        }
        Ok(csrf)
    }

    fn delete_by_key<T: AsRef<str>>(conn: &PgConnection, key: T) -> DriverResult<()> {
        diesel::delete(sso_csrf::table.filter(sso_csrf::dsl::key.eq(key.as_ref())))
            .execute(conn)
            .map_err(Into::into)
            .map(|_| ())
    }

    fn delete_by_ttl(conn: &PgConnection) -> DriverResult<()> {
        let now = Utc::now();
        diesel::delete(sso_csrf::table.filter(sso_csrf::dsl::ttl.le(now)))
            .execute(conn)
            .map_err(Into::into)
            .map(|_| ())
    }

    /// Returns error if service ID does not match CSRF token.
    fn check_service(&self, service_id: Uuid) -> DriverResult<()> {
        if service_id != self.service_id {
            Err(DriverError::CsrfServiceMismatch)
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for Csrf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Csrf {}", self.key)?;
        write!(f, "\n\tcreated_at {}", self.created_at)?;
        write!(f, "\n\tvalue {}", self.value)?;
        write!(f, "\n\tttl {}", self.ttl)?;
        write!(f, "\n\tservice_id {}", self.service_id)
    }
}

impl From<Csrf> for pb::Csrf {
    fn from(r: Csrf) -> Self {
        Self {
            created_at: util::datetime_to_timestamp_opt(r.created_at),
            key: r.key,
            value: r.value,
            ttl: util::datetime_to_timestamp_opt(r.ttl),
            service_id: Some(util::uuid_to_string(r.service_id)),
        }
    }
}

/// CSRF create.
#[derive(Debug, Insertable)]
#[table_name = "sso_csrf"]
pub struct CsrfCreate {
    created_at: DateTime<Utc>,
    key: String,
    value: String,
    ttl: DateTime<Utc>,
    service_id: Uuid,
}

impl CsrfCreate {
    /// Generate random CSRF token with time to live in seconds.
    pub fn generate(ttl_s: i64, service_id: Uuid) -> Self {
        let key = KeyBuilder::new()
            .size(CSRF_KEY_BYTES)
            .generate()
            .as_base32();
        Self::new(&key, &key, ttl_s, service_id)
    }

    /// Create CSRF token with time to live in seconds. Key must be unique.
    pub fn new<K, V>(key: K, value: V, ttl_s: i64, service_id: Uuid) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            created_at: Utc::now(),
            key: key.into(),
            value: value.into(),
            ttl: Utc::now() + Duration::seconds(ttl_s),
            service_id,
        }
    }
}
