use crate::{
    client_msg::Get, AuditBuilder, ClientActor, Core, CoreError, CoreResult, Driver, Service,
};
use actix::Addr;
use chrono::{DateTime, Utc};
use futures::{future, Future};
use libreauth::pass::HashBuilder;
use sha1::{Digest, Sha1};
use std::fmt;
use uuid::Uuid;

/// User name maximum length.
pub const USER_NAME_MAX_LEN: usize = 100;

/// User password hash version passed to libreauth hash builder.
pub const USER_PASSWORD_HASH_VERSION: usize = 1;

/// User password minimum length.
pub const USER_PASSWORD_MIN_LEN: usize = 8;

/// User password maximum length.
pub const USER_PASSWORD_MAX_LEN: usize = 128;

/// User.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: Uuid,
    pub is_enabled: bool,
    pub name: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: Option<String>,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User {}", self.id)?;
        write!(f, "\n\tcreated_at {}", self.created_at)?;
        write!(f, "\n\tupdated_at {}", self.updated_at)?;
        write!(f, "\n\tis_enabled {}", self.is_enabled)?;
        write!(f, "\n\tname {}", self.name)?;
        write!(f, "\n\temail {}", self.email)
    }
}

/// User password metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserPasswordMeta {
    pub password_strength: Option<u8>,
    pub password_pwned: Option<bool>,
}

impl Default for UserPasswordMeta {
    fn default() -> Self {
        UserPasswordMeta {
            password_strength: None,
            password_pwned: None,
        }
    }
}

/// User create data.
pub struct UserCreate<'a> {
    pub is_enabled: bool,
    pub name: &'a str,
    pub email: &'a str,
    pub password_hash: Option<&'a str>,
}

/// User update data.
pub struct UserUpdate<'a> {
    pub is_enabled: Option<bool>,
    pub name: Option<&'a str>,
}

/// User query.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserQuery {
    pub gt: Option<Uuid>,
    pub lt: Option<Uuid>,
    pub limit: Option<i64>,
    pub email_eq: Option<String>,
}

/// User token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserToken {
    pub user_id: Uuid,
    pub access_token: String,
    pub access_token_expires: i64,
    pub refresh_token: String,
    pub refresh_token_expires: i64,
}

/// User access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTokenAccess {
    pub user_id: Uuid,
    pub access_token: String,
    pub access_token_expires: i64,
}

/// User key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserKey {
    pub user_id: Uuid,
    pub key: String,
}

impl User {
    /// List users using query.
    pub fn list(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        query: &UserQuery,
    ) -> CoreResult<Vec<Uuid>> {
        let limit = query.limit.unwrap_or_else(Core::default_limit);

        if let Some(email_eq) = &query.email_eq {
            let users = driver
                .user_list_where_email_eq(email_eq, limit)
                .map_err(CoreError::Driver)?;
            return Ok(users);
        }

        match &query.gt {
            Some(gt) => driver
                .user_list_where_id_gt(*gt, limit)
                .map_err(CoreError::Driver),
            None => match &query.lt {
                Some(lt) => driver
                    .user_list_where_id_lt(*lt, limit)
                    .map_err(CoreError::Driver),
                None => driver
                    .user_list_where_id_gt(Uuid::nil(), limit)
                    .map_err(CoreError::Driver),
            },
        }
    }

    /// Create user.
    /// Returns bad request if email address is not unique.
    pub fn create(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        audit: &mut AuditBuilder,
        is_enabled: bool,
        name: &str,
        email: &str,
        password: Option<&str>,
    ) -> CoreResult<User> {
        let user = User::read_by_email(driver, service_mask, audit, email)?;
        if user.is_some() {
            return Err(CoreError::BadRequest);
        }

        let password_hash = User::password_hash(password)?;
        let create = UserCreate {
            is_enabled,
            name,
            email,
            password_hash: password_hash.as_ref().map(|x| &**x),
        };
        driver.user_create(&create).map_err(CoreError::Driver)
    }

    /// Read user by ID.
    pub fn read_by_id(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
    ) -> CoreResult<Option<User>> {
        driver.user_read_by_id(id).map_err(CoreError::Driver)
    }

    /// Read user by email.
    pub fn read_by_email(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        email: &str,
    ) -> CoreResult<Option<User>> {
        driver.user_read_by_email(email).map_err(CoreError::Driver)
    }

    /// Update user by ID.
    pub fn update_by_id(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> CoreResult<User> {
        let update = UserUpdate { is_enabled, name };
        driver
            .user_update_by_id(id, &update)
            .map_err(CoreError::Driver)
    }

    /// Update user email by ID.
    pub fn update_email_by_id(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
        email: &str,
    ) -> CoreResult<usize> {
        driver
            .user_update_email_by_id(id, email)
            .map_err(CoreError::Driver)
    }

    /// Update user password by ID.
    pub fn update_password_by_id(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
        password: &str,
    ) -> CoreResult<usize> {
        let password_hash =
            User::password_hash(Some(password))?.ok_or_else(|| CoreError::Forbidden)?;
        driver
            .user_update_password_by_id(id, &password_hash)
            .map_err(CoreError::Driver)
    }

    /// Delete user by ID.
    pub fn delete_by_id(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
    ) -> CoreResult<usize> {
        driver.user_delete_by_id(id).map_err(CoreError::Driver)
    }

    /// Hash password string, none is returned if none is given as the input.
    /// <https://github.com/breard-r/libreauth>
    pub fn password_hash(password: Option<&str>) -> CoreResult<Option<String>> {
        match password {
            Some(password) => {
                let hasher = HashBuilder::new()
                    .version(USER_PASSWORD_HASH_VERSION)
                    .min_len(USER_PASSWORD_MIN_LEN)
                    .max_len(USER_PASSWORD_MAX_LEN)
                    .finalize()
                    .map_err(CoreError::libreauth_pass)?;

                let hashed = hasher.hash(password).map_err(CoreError::libreauth_pass)?;
                Ok(Some(hashed))
            }
            None => Ok(None),
        }
    }

    /// Check if password string and password hash match, an error is returned if they do not match or the hash is none.
    /// Returns true if the hash version does not match the current hash version.
    pub fn password_check(password_hash: Option<&str>, password: &str) -> CoreResult<bool> {
        match password_hash {
            Some(password_hash) => {
                let checker =
                    HashBuilder::from_phc(password_hash).map_err(CoreError::libreauth_pass)?;

                if checker.is_valid(password) {
                    Ok(checker.needs_update(Some(USER_PASSWORD_HASH_VERSION)))
                } else {
                    Err(CoreError::BadRequest)
                }
            }
            None => Err(CoreError::BadRequest),
        }
    }

    /// Returns password strength and pwned checks.
    pub fn password_meta(
        enabled: bool,
        client: &Addr<ClientActor>,
        password: Option<&str>,
    ) -> impl Future<Item = UserPasswordMeta, Error = CoreError> {
        match password {
            Some(password) => {
                let password_strength = User::password_meta_strength(password).then(|r| match r {
                    Ok(entropy) => future::ok(Some(entropy.score)),
                    Err(err) => {
                        warn!("{}", err);
                        future::ok(None)
                    }
                });
                let password_pwned =
                    User::password_meta_pwned(enabled, client, password).then(|r| match r {
                        Ok(password_pwned) => future::ok(Some(password_pwned)),
                        Err(err) => {
                            warn!("{}", err);
                            future::ok(None)
                        }
                    });
                future::Either::A(password_strength.join(password_pwned).map(
                    |(password_strength, password_pwned)| UserPasswordMeta {
                        password_strength,
                        password_pwned,
                    },
                ))
            }
            None => future::Either::B(future::ok(UserPasswordMeta::default())),
        }
    }

    /// Returns password strength test performed by `zxcvbn`.
    /// <https://github.com/shssoichiro/zxcvbn-rs>
    fn password_meta_strength(
        password: &str,
    ) -> impl Future<Item = zxcvbn::Entropy, Error = CoreError> {
        future::result(zxcvbn::zxcvbn(password, &[]).map_err(CoreError::Zxcvbn))
    }

    /// Returns true if password is present in `Pwned Passwords` index, else false.
    /// <https://haveibeenpwned.com/Passwords>
    fn password_meta_pwned(
        enabled: bool,
        client: &Addr<ClientActor>,
        password: &str,
    ) -> impl Future<Item = bool, Error = CoreError> {
        if enabled {
            // Make request to API using first 5 characters of SHA1 password hash.
            let mut hash = Sha1::new();
            hash.input(password);
            let hash = format!("{:X}", hash.result());
            let route = format!("/range/{:.5}", hash);

            future::Either::A(
                // Make API request.
                client
                    .send(Get::new("https://api.pwnedpasswords.com", route))
                    .map_err(CoreError::ActixMailbox)
                    .and_then(|res| res.map_err(CoreError::Client))
                    // Compare suffix of hash to lines to determine if password is pwned.
                    .and_then(move |text| {
                        for line in text.lines() {
                            if hash[5..] == line[..35] {
                                return Ok(true);
                            }
                        }
                        Ok(false)
                    }),
            )
        } else {
            future::Either::B(future::err(CoreError::PwnedPasswordsDisabled))
        }
    }
}
