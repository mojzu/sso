use crate::{prelude::*, schema::sso_csrf};
use diesel::{prelude::*, PgConnection};
use libreauth::key::KeyBuilder;
use std::fmt;

/// CSRF key size in bytes.
const CSRF_KEY_BYTES: usize = 11;

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

/// CSRF verify.
#[derive(Debug)]
pub struct CsrfVerify(Uuid, Option<String>);

/// CSRF read.
#[derive(Debug)]
pub struct CsrfRead(String);

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
            created_at: pb::datetime_to_timestamp_opt(r.created_at),
            key: r.key,
            value: r.value,
            ttl: pb::datetime_to_timestamp_opt(r.ttl),
            service_id: Some(pb::uuid_to_string(r.service_id)),
        }
    }
}

impl validator::Validate for pb::AuthCsrfCreateRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        validate::wrap(|e| {
            validate::csrf_expires_s_opt(e, "expires_s", self.expires_s);
        })
    }
}

impl validator::Validate for pb::AuthCsrfVerifyRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        validate::wrap(|e| {
            validate::csrf_token(e, "csrf", &self.csrf);
            validate::audit_type_opt(e, "audit", self.audit.as_ref().map(|x| &**x))
        })
    }
}

impl CsrfCreate {
    /// Create from request.
    pub fn request(
        conn: &PgConnection,
        req: &pb::AuthCsrfCreateRequest,
        service_id: Uuid,
    ) -> DriverResult<pb::Csrf> {
        let expires_s = req.expires_s.unwrap_or(DEFAULT_CSRF_EXPIRES_S);
        let expires = Duration::seconds(expires_s);
        Self::generate(conn, expires, service_id).map(Into::into)
    }

    /// Generate random CSRF key with time to live for service.
    pub fn generate(conn: &PgConnection, ttl: Duration, service_id: Uuid) -> DriverResult<Csrf> {
        let key = KeyBuilder::new()
            .size(CSRF_KEY_BYTES)
            .generate()
            .as_base32();
        Self::create(conn, &key, &key, ttl, service_id)
    }

    /// Create CSRF key/value with time to live for service. Key must be unique.
    pub fn create<K, V>(
        conn: &PgConnection,
        key: K,
        value: V,
        ttl: Duration,
        service_id: Uuid,
    ) -> DriverResult<Csrf>
    where
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            created_at: Utc::now(),
            key: key.into(),
            value: value.into(),
            ttl: Utc::now() + ttl,
            service_id,
        }
        .create_inner(conn)
    }

    /// Insert CSRF model in database.
    fn create_inner(&self, conn: &PgConnection) -> DriverResult<Csrf> {
        diesel::insert_into(sso_csrf::table)
            .values(self)
            .get_result::<Csrf>(conn)
            .map_err(Into::into)
    }
}

impl CsrfVerify {
    /// Verify from request.
    pub fn request(
        conn: &PgConnection,
        req: &pb::AuthCsrfVerifyRequest,
        service_id: Uuid,
    ) -> DriverResult<pb::Csrf> {
        Self::verify(conn, service_id, Some(req.csrf.clone())).map(Into::into)
    }

    /// Verify a CSRF key for service ID, fails if service did not create key.
    pub fn verify(
        conn: &PgConnection,
        service_id: Uuid,
        csrf_key: Option<String>,
    ) -> DriverResult<Csrf> {
        let csrf_key = csrf_key.ok_or_else(|| DriverError::CsrfNotFoundOrUsed)?;
        CsrfRead::read(conn, &csrf_key)?
            .ok_or_else(|| DriverError::CsrfNotFoundOrUsed)
            .and_then(|csrf| {
                if service_id != csrf.service_id {
                    Err(DriverError::CsrfServiceMismatch)
                } else {
                    Ok(csrf)
                }
            })
    }
}

impl CsrfRead {
    /// Read CSRF token. CSRF token is deleted after one read.
    pub fn read<T: AsRef<str>>(conn: &PgConnection, key: T) -> DriverResult<Option<Csrf>> {
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
}
