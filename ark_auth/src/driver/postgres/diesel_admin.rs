//! # System Administration Functions
//! <https://www.postgresql.org/docs/9.3/functions-admin.html>

mod types {
    #[derive(Clone, Copy, SqlType)]
    #[postgres(oid = "2278", array_oid = "0")]
    pub struct Void;
}

mod functions {
    use super::types::*;
    use diesel::sql_types::*;

    // Advisory Lock Functions
    // Obtain exclusive transaction level advisory lock.
    sql_function!(pg_advisory_xact_lock, pg_advisory_xact_lock_t, (key1: Integer, key2: Integer) -> Void);
    // Obtain shared transaction level advisory lock.
    sql_function!(pg_advisory_xact_lock_shared, pg_advisory_xact_lock_shared_t, (key1: Integer, key2: Integer) -> Void);
    // Obtain exclusive transaction level advisory lock if available.
    sql_function!(pg_try_advisory_xact_lock, pg_try_advisory_xact_lock_t, (key1: Integer, key2: Integer) -> Bool);
    // Obtain shared transaction level advisory lock if available.
    sql_function!(pg_try_advisory_xact_lock_shared, pg_try_advisory_xact_lock_shared_t, (key1: Integer, key2: Integer) -> Bool);
}

pub use self::functions::*;
pub use self::types::*;
