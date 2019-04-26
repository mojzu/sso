use crate::db::{DbError, DbOrder};
use crate::models::{AuthUser, AuthUserInsert, AuthUserUpdate};
use crate::schema;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;

pub fn list(
    offset: i64,
    limit: i64,
    order: DbOrder,
    conn: &PgConnection,
) -> Result<Vec<AuthUser>, DbError> {
    use crate::schema::auth_user::dsl::*;

    let filter_expr: Box<BoxableExpression<schema::auth_user::table, Pg, SqlType = Bool>> =
        match order {
            DbOrder::Asc => Box::new(user_id.gt(offset)),
            DbOrder::Desc => Box::new(user_id.lt(offset)),
        };
    let order_expr: Box<BoxableExpression<schema::auth_user::table, Pg, SqlType = ()>> = match order
    {
        DbOrder::Asc => Box::new(user_id.asc()),
        DbOrder::Desc => Box::new(user_id.desc()),
    };
    auth_user
        .filter(filter_expr)
        .limit(limit)
        .order(order_expr)
        .load::<AuthUser>(conn)
        .map_err(Into::into)
}

pub fn create(
    name: &str,
    email: &str,
    password: Option<&str>,
    conn: &PgConnection,
) -> Result<AuthUser, DbError> {
    use crate::schema::auth_user::dsl::*;

    let (password_hash, password_pwned) = hash_password(password)?;
    let value = AuthUserInsert {
        user_name: name,
        user_email: email,
        user_password: password_hash.as_ref().map(|x| &**x),
        user_password_revision: 1,
        user_password_pwned: password_pwned,
    };
    diesel::insert_into(auth_user)
        .values(&value)
        .get_result::<AuthUser>(conn)
        .map_err(Into::into)
}

pub fn read_by_id(id: i64, conn: &PgConnection) -> Result<AuthUser, DbError> {
    use crate::schema::auth_user::dsl::*;

    auth_user
        .filter(user_id.eq(id))
        .get_result::<AuthUser>(conn)
        .map_err(Into::into)
}

pub fn read_by_email(email: &str, conn: &PgConnection) -> Result<AuthUser, DbError> {
    use crate::schema::auth_user::dsl::*;

    auth_user
        .filter(user_email.eq(email))
        .get_result::<AuthUser>(conn)
        .map_err(Into::into)
}

pub fn update_by_id(id: i64, name: Option<&str>, conn: &PgConnection) -> Result<AuthUser, DbError> {
    use crate::schema::auth_user::dsl::*;

    let user_updated_at = chrono::Utc::now();
    let value = AuthUserUpdate {
        updated_at: &user_updated_at,
        user_name: name,
    };
    diesel::update(auth_user.filter(user_id.eq(id)))
        .set(&value)
        .get_result::<AuthUser>(conn)
        .map_err(Into::into)
}

pub fn update_password_by_id(
    id: i64,
    password: &str,
    password_revision: i32,
    conn: &PgConnection,
) -> Result<usize, DbError> {
    use crate::schema::auth_user::dsl::*;

    let user = read_by_id(id, conn)?;
    if password_revision != user.user_password_revision {
        return Err(DbError::InvalidPasswordRevision);
    }

    let user_updated_at = chrono::Utc::now();
    let (password_hash, password_pwned) = hash_password(Some(password))?;
    diesel::update(auth_user.filter(user_id.eq(id)))
        .set((
            updated_at.eq(user_updated_at),
            user_password.eq(password_hash),
            user_password_revision.eq(user.user_password_revision + 1),
            user_password_pwned.eq(password_pwned),
        ))
        .execute(conn)
        .map_err(Into::into)
}

pub fn delete_by_id(id: i64, conn: &PgConnection) -> Result<usize, DbError> {
    use crate::schema::auth_user::dsl::*;

    diesel::delete(auth_user.filter(user_id.eq(id)))
        .execute(conn)
        .map_err(Into::into)
}

/// Hash password string using bcrypt and checks haveibeenpwned.com for leaked passwords.
/// If password is None, a random password is generated and hashed.
pub fn hash_password(password: Option<&str>) -> Result<(String, bool), DbError> {
    match password {
        Some(password) => {
            let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(DbError::Bcrypt)?;
            Ok((hashed, false))
        }
        None => Ok((None, false)),
    }
}

/// Checks if password string and password bcrypt hash are equal.
pub fn check_password(user_password: Option<&str>, check_password: &str) -> Result<(), DbError> {
    match user_password {
        Some(user_password) => bcrypt::verify(check_password, user_password)
            .map_err(DbError::Bcrypt)
            .and_then(|verified| {
                if verified {
                    Ok(())
                } else {
                    Err(DbError::InvalidPassword)
                }
            }),
        None => Err(DbError::InvalidPassword),
    }
}

// TODO(feature): Pwned password check in hash_password.
// const client = this.restifyClients.createStringClient(
//     `https://api.pwnedpasswords.com/range/${sha1Hash.substr(0, 5)}`,
// );
// const response = await client.get("");
// const index: { [key: string]: number } = {};
// if (response.data != null) {
//     response.data.split("\r\n").map((line) => {
//         const [hash, count] = line.split(":");
//         index[hash.trim()] = Number(count.trim());
//     });
// }
// return has(index, sha1Hash.toUpperCase().substring(5));
