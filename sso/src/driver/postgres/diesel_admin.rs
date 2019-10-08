//! # System Administration Functions
//! <https://www.postgresql.org/docs/9.3/functions-admin.html>

mod types {
    #[derive(Debug, Clone, Copy, SqlType)]
    #[postgres(oid = "2278", array_oid = "0")]
    #[sqlite_type = "Integer"]
    pub struct Void;

    // impl HasSqlType<Void> for Pg {
    //     fn metadata(_: &PgMetadataLookup) -> PgTypeMetadata {
    //         PgTypeMetadata {
    //             oid: 2278,
    //             array_oid: 0,
    //         }
    //     }
    // }
}

mod functions {
    use super::types::*;
    use diesel::sql_types::*;

    sql_function! {
        /// Obtain exclusive transaction level advisory lock.
        fn pg_advisory_xact_lock(key1: Integer, key2: Integer) -> Void;
    }
    sql_function! {
        /// Obtain shared transaction level advisory lock.
        fn pg_advisory_xact_lock_shared(key1: Integer, key2: Integer) -> Void;
    }
    sql_function! {
        /// Obtain exclusive transaction level advisory lock if available.
        fn pg_try_advisory_xact_lock(key1: Integer, key2: Integer) -> Bool;
    }
    sql_function! {
        /// Obtain shared transaction level advisory lock if available.
        fn pg_try_advisory_xact_lock_shared(key1: Integer, key2: Integer) -> Bool;
    }
}

// mod helper_types {
//     use super::functions;

//     /// The return type of `pg_advisory_xact_lock(expr, expr)`.
//     pub type PgAdvisoryXactLock<Expr1, Expr2> =
//         functions::pg_advisory_xact_lock::HelperType<Expr1, Expr2>;

//     /// The return type of `pg_advisory_xact_lock_shared(expr, expr)`.
//     pub type PgAdvisoryXactLockShared<Expr1, Expr2> =
//         functions::pg_advisory_xact_lock_shared::HelperType<Expr1, Expr2>;

//     /// The return type of `pg_try_advisory_xact_lock(expr, expr)`.
//     pub type PgTryAdvisoryXactLock<Expr1, Expr2> =
//         functions::pg_try_advisory_xact_lock::HelperType<Expr1, Expr2>;

//     /// The return type of `pg_try_advisory_xact_lock_shared(expr, expr)`.
//     pub type PgTryAdvisoryXactLockShared<Expr1, Expr2> =
//         functions::pg_try_advisory_xact_lock_shared::HelperType<Expr1, Expr2>;
// }

pub use functions::*;
pub use types::*;
