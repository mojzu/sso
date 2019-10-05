use crate::{client_msg::Get, AuditBuilder, ClientActor, CoreError, CoreResult, Driver, Service};
use actix::Addr;
use chrono::{DateTime, Utc};
use futures::{future, Future};
use libreauth::pass::HashBuilder;
use sha1::{Digest, Sha1};
use std::fmt;
use uuid::Uuid;

/// User name maximum length.
pub const USER_NAME_MAX_LEN: usize = 100;

/// User locale maximum length.
pub const USER_LOCALE_MAX_LEN: usize = 10;

/// User timezone maximum length.
pub const USER_TIMEZONE_MAX_LEN: usize = 50;

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
    pub locale: String,
    pub timezone: String,
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub password_update_required: Option<bool>,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User {}", self.id)?;
        write!(f, "\n\tcreated_at {}", self.created_at)?;
        write!(f, "\n\tupdated_at {}", self.updated_at)?;
        write!(f, "\n\tis_enabled {}", self.is_enabled)?;
        write!(f, "\n\tname {}", self.name)?;
        write!(f, "\n\temail {}", self.email)?;
        write!(f, "\n\tlocale {}", self.locale)?;
        write!(f, "\n\ttimezone {}", self.timezone)?;
        if let Some(password_update_required) = self.password_update_required {
            write!(
                f,
                "\n\tpassword_update_required {}",
                password_update_required
            )?;
        }
        Ok(())
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

/// User list.
#[derive(Debug)]
pub enum UserList {
    Limit(i64),
    IdGt(Uuid, i64),
    IdLt(Uuid, i64),
    EmailEq(String, i64),
}

/// User create.
pub struct UserCreate {
    pub is_enabled: bool,
    pub name: String,
    pub email: String,
    pub locale: String,
    pub timezone: String,
    pub password_hash: Option<String>,
    pub password_update_required: Option<bool>,
}

/// User read.
#[derive(Debug)]
pub enum UserRead {
    Id(Uuid),
    Email(String),
}

/// User update.
pub struct UserUpdate {
    pub is_enabled: Option<bool>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub password_hash: Option<String>,
    pub password_update_required: Option<bool>,
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
    /// List usersy.
    pub fn list(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        list: &UserList,
    ) -> CoreResult<Vec<User>> {
        driver.user_list(list).map_err(CoreError::Driver)
    }

    /// Create user.
    /// Returns bad request if email address is not unique.
    pub fn create(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        audit: &mut AuditBuilder,
        is_enabled: bool,
        name: String,
        email: String,
        locale: String,
        timezone: String,
        password: Option<String>,
        password_update_required: Option<bool>,
    ) -> CoreResult<User> {
        let read = UserRead::Email(email.clone());
        let user = User::read_opt(driver, service_mask, audit, &read)?;
        if user.is_some() {
            return Err(CoreError::BadRequest);
        }

        let password_hash = User::password_hash(password.as_ref().map(|x| &**x))?;
        let create = UserCreate {
            is_enabled,
            name,
            email,
            locale,
            timezone,
            password_hash,
            password_update_required,
        };
        driver.user_create(&create).map_err(CoreError::Driver)
    }

    /// Read user (optional).
    pub fn read_opt(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        read: &UserRead,
    ) -> CoreResult<Option<User>> {
        driver.user_read_opt(read).map_err(CoreError::Driver)
    }

    /// Update user.
    pub fn update(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
        is_enabled: Option<bool>,
        name: Option<String>,
        locale: Option<String>,
        timezone: Option<String>,
        password_update_required: Option<bool>,
    ) -> CoreResult<User> {
        let update = UserUpdate {
            is_enabled,
            name,
            email: None,
            locale,
            timezone,
            password_hash: None,
            password_update_required,
        };
        driver.user_update(&id, &update).map_err(CoreError::Driver)
    }

    /// Update user email by ID.
    pub fn update_email(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
        email: String,
    ) -> CoreResult<User> {
        let update = UserUpdate {
            is_enabled: None,
            name: None,
            email: Some(email),
            locale: None,
            timezone: None,
            password_hash: None,
            password_update_required: None,
        };
        driver.user_update(&id, &update).map_err(CoreError::Driver)
    }

    /// Update user password by ID.
    pub fn update_password(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
        password: String,
    ) -> CoreResult<User> {
        let password_hash =
            User::password_hash(Some(&password))?.ok_or_else(|| CoreError::Forbidden)?;
        let update = UserUpdate {
            is_enabled: None,
            name: None,
            email: None,
            locale: None,
            timezone: None,
            password_hash: Some(password_hash),
            password_update_required: None,
        };
        driver.user_update(&id, &update).map_err(CoreError::Driver)
    }

    /// Delete user.
    pub fn delete(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
    ) -> CoreResult<usize> {
        driver.user_delete(&id).map_err(CoreError::Driver)
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
