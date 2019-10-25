/// Driver errors.
#[derive(Debug, Fail)]
pub enum DriverError {
    #[fail(display = "KeyUserTokenConstraint")]
    KeyUserTokenConstraint,

    #[fail(display = "KeyUserTotpConstraint")]
    KeyUserTotpConstraint,

    #[fail(display = "ServiceNotFound")]
    ServiceNotFound,

    #[fail(display = "ServiceDisabled")]
    ServiceDisabled,

    #[fail(display = "ServiceProviderLocalDisabled")]
    ServiceProviderLocalDisabled,

    #[fail(display = "UserNotFound")]
    UserNotFound,

    #[fail(display = "UserEmailConstraint")]
    UserEmailConstraint,

    #[fail(display = "UserPasswordIncorrect")]
    UserPasswordIncorrect,

    #[fail(display = "UserPasswordUndefined")]
    UserPasswordUndefined,

    #[fail(display = "CsrfServiceMismatch")]
    CsrfServiceMismatch,

    #[fail(display = "Locked {:?}", _0)]
    Locked(i32),

    #[fail(display = "LockFn {}", _0)]
    LockFn(String),

    #[fail(display = "Delete")]
    Delete,

    #[fail(display = "DieselResult {}", _0)]
    DieselResult(#[fail(cause)] diesel::result::Error),

    #[fail(display = "DieselMigrations {}", _0)]
    DieselMigrations(#[fail(cause)] diesel_migrations::RunMigrationsError),

    #[fail(display = "LibreauthPass {}", _0)]
    LibreauthPass(usize),

    #[fail(display = "R2d2 {}", _0)]
    R2d2(#[fail(cause)] r2d2::Error),

    #[fail(display = "Rustls")]
    Rustls,

    #[fail(display = "StdIo {}", _0)]
    StdIo(#[fail(cause)] std::io::Error),

    #[fail(display = "Prometheus {}", _0)]
    Prometheus(#[fail(cause)] prometheus::Error),

    #[fail(display = "SerdeJson {}", _0)]
    SerdeJson(#[fail(cause)] serde_json::Error),

    #[fail(display = "SerdeQs {}", _0)]
    SerdeQs(String),

    #[fail(display = "Metrics")]
    Metrics,
}

impl DriverError {
    pub fn libreauth_pass(e: libreauth::pass::ErrorCode) -> Self {
        Self::LibreauthPass(e as usize)
    }

    pub fn serde_qs(e: serde_qs::Error) -> Self {
        Self::SerdeQs(e.description().to_owned())
    }
}

impl From<diesel::result::Error> for DriverError {
    fn from(e: diesel::result::Error) -> Self {
        Self::DieselResult(e)
    }
}

impl From<diesel_migrations::RunMigrationsError> for DriverError {
    fn from(e: diesel_migrations::RunMigrationsError) -> Self {
        Self::DieselMigrations(e)
    }
}

impl From<r2d2::Error> for DriverError {
    fn from(e: r2d2::Error) -> Self {
        Self::R2d2(e)
    }
}

/// Driver result wrapper type.
pub type DriverResult<T> = Result<T, DriverError>;
