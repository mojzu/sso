use crate::{
    client_msg::Get, AuditDiff, AuditDiffBuilder, AuditSubject, Auth, ClientActor, CoreError,
    CoreResult, Driver, Service,
};
use actix::Addr;
use chrono::{DateTime, Utc};
use futures::{future, Future};
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::fmt;
use uuid::Uuid;

/// User name maximum length.
pub const USER_NAME_MAX_LEN: usize = 100;

/// User locale maximum length.
pub const USER_LOCALE_MAX_LEN: usize = 10;

/// User timezone maximum length.
pub const USER_TIMEZONE_MAX_LEN: usize = 50;

/// User password hash version.
///
/// Passed to libreauth hash builder.
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
    pub password_allow_reset: bool,
    pub password_require_update: bool,
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
        write!(f, "\n\temail {}", self.email)?;
        write!(f, "\n\tlocale {}", self.locale)?;
        write!(f, "\n\ttimezone {}", self.timezone)?;
        write!(f, "\n\tpassword_allow_reset {}", self.password_allow_reset)?;
        write!(
            f,
            "\n\tpassword_require_update {}",
            self.password_require_update
        )
    }
}

impl AuditSubject for User {
    fn subject(&self) -> String {
        format!("{}", self.id)
    }
}

impl AuditDiff for User {
    fn diff(&self, previous: &Self) -> Value {
        AuditDiffBuilder::default()
            .compare("is_enabled", &self.is_enabled, &previous.is_enabled)
            .compare("name", &self.name, &previous.name)
            .compare("email", &self.email, &previous.email)
            .compare("locale", &self.locale, &previous.locale)
            .compare("timezone", &self.timezone, &previous.timezone)
            .compare(
                "password_allow_reset",
                &self.password_allow_reset,
                &previous.password_allow_reset,
            )
            .compare(
                "password_require_update",
                &self.password_require_update,
                &previous.password_require_update,
            )
            .into_value()
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

/// User list query.
#[derive(Debug)]
pub enum UserListQuery {
    /// Where ID greater than.
    IdGt(Uuid, i64),
    /// Where ID less than.
    IdLt(Uuid, i64),
    /// Where name greater than or equal.
    NameGe(String, i64, Option<Uuid>),
    /// Where name less than or equal.
    NameLe(String, i64, Option<Uuid>),
}

/// User list filter.
#[derive(Debug)]
pub struct UserListFilter {
    pub id: Option<Vec<Uuid>>,
    pub email: Option<Vec<String>>,
}

/// User list.
#[derive(Debug)]
pub struct UserList<'a> {
    pub query: &'a UserListQuery,
    pub filter: &'a UserListFilter,
}

/// User create.
#[derive(Debug)]
pub struct UserCreate {
    pub is_enabled: bool,
    pub name: String,
    pub email: String,
    pub locale: String,
    pub timezone: String,
    pub password_allow_reset: bool,
    pub password_require_update: bool,
    pub password_hash: Option<String>,
}

/// User read.
#[derive(Debug)]
pub enum UserRead {
    Id(Uuid),
    Email(String),
}

/// User update.
#[derive(Debug)]
pub struct UserUpdate {
    pub is_enabled: Option<bool>,
    pub name: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub password_allow_reset: Option<bool>,
    pub password_require_update: Option<bool>,
}

/// User update 2.
///
/// This is split from `UserUpdate` to prevent `User::update_email` or
/// `User::update_password` functions being bypassed which could
/// allow an unhashed password to be saved to the database.
#[derive(Debug)]
pub struct UserUpdate2 {
    pub email: Option<String>,
    pub password_hash: Option<String>,
}

/// User token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserToken {
    pub user: User,
    pub access_token: String,
    pub access_token_expires: i64,
    pub refresh_token: String,
    pub refresh_token_expires: i64,
}

/// User access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTokenAccess {
    pub user: User,
    pub access_token: String,
    pub access_token_expires: i64,
}

/// User key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserKey {
    pub user: User,
    pub key: String,
}

impl User {
    pub fn password_hash(&self) -> Option<&str> {
        self.password_hash.as_ref().map(|x| &**x)
    }

    /// Create user.
    /// Returns error if email address is not unique.
    pub fn create(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        create: &mut UserCreate,
    ) -> CoreResult<User> {
        let read = UserRead::Email(create.email.clone());
        let user = User::read_opt(driver, service_mask, &read)?;
        if user.is_some() {
            return Err(CoreError::UserEmailConstraint);
        }

        create.password_hash = Auth::password_hash(create.password_hash.as_ref().map(|x| &**x))?;
        driver.user_create(create).map_err(CoreError::Driver)
    }

    /// Read user.
    pub fn read(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        read: &UserRead,
    ) -> CoreResult<User> {
        Self::read_opt(driver, service_mask, read)?.ok_or_else(|| CoreError::UserNotFound)
    }

    /// Read user (optional).
    pub fn read_opt(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        read: &UserRead,
    ) -> CoreResult<Option<User>> {
        driver.user_read(read).map_err(CoreError::Driver)
    }

    /// Update user.
    pub fn update(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        id: Uuid,
        update: &UserUpdate,
    ) -> CoreResult<User> {
        driver.user_update(&id, update).map_err(CoreError::Driver)
    }

    /// Update user email by ID.
    pub fn update_email(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        id: Uuid,
        email: String,
    ) -> CoreResult<User> {
        let update = UserUpdate2 {
            email: Some(email),
            password_hash: None,
        };
        driver.user_update2(&id, &update).map_err(CoreError::Driver)
    }

    /// Update user password by ID.
    pub fn update_password(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        id: Uuid,
        password: String,
    ) -> CoreResult<User> {
        let password_hash = Auth::password_hash(Some(&password))?.unwrap();
        let update = UserUpdate2 {
            email: None,
            password_hash: Some(password_hash),
        };
        driver.user_update2(&id, &update).map_err(CoreError::Driver)
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
        // TODO(fix): Fix "Zxcvbn cannot evaluate a blank password" warning.
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
