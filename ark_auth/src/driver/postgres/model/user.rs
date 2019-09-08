use crate::{driver::postgres::schema::auth_user, DriverError};
use chrono::{DateTime, Utc};
use diesel::result::Error as DieselError;
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_user"]
#[primary_key(user_id)]
pub struct User {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: Uuid,
    pub user_is_enabled: bool,
    pub user_name: String,
    pub user_email: String,
    pub user_password_hash: Option<String>,
}

#[derive(Debug, Insertable)]
#[table_name = "auth_user"]
pub struct UserInsert<'a> {
    pub created_at: &'a DateTime<Utc>,
    pub updated_at: &'a DateTime<Utc>,
    pub user_id: Uuid,
    pub user_is_enabled: bool,
    pub user_name: &'a str,
    pub user_email: &'a str,
    pub user_password_hash: Option<&'a str>,
}

#[derive(AsChangeset)]
#[table_name = "auth_user"]
pub struct UserUpdate<'a> {
    pub updated_at: &'a DateTime<Utc>,
    pub user_is_enabled: Option<bool>,
    pub user_name: Option<&'a str>,
}

#[derive(AsChangeset)]
#[table_name = "auth_user"]
pub struct UserUpdatePassword<'a> {
    pub updated_at: &'a DateTime<Utc>,
    pub user_password_hash: String,
}

impl User {
    pub fn list_where_id_lt(
        conn: &PgConnection,
        lt: Uuid,
        limit: i64,
    ) -> Result<Vec<Uuid>, DriverError> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        auth_user
            .select(user_id)
            .filter(user_id.lt(lt))
            .limit(limit)
            .order(user_id.desc())
            .load::<Uuid>(conn)
            .map_err(DriverError::Diesel)
            .map(|mut v| {
                v.reverse();
                v
            })
    }

    pub fn list_where_id_gt(
        conn: &PgConnection,
        gt: Uuid,
        limit: i64,
    ) -> Result<Vec<Uuid>, DriverError> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        auth_user
            .select(user_id)
            .filter(user_id.gt(gt))
            .limit(limit)
            .order(user_id.asc())
            .load::<Uuid>(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn list_where_email_eq(
        conn: &PgConnection,
        email_eq: &str,
        limit: i64,
    ) -> Result<Vec<Uuid>, DriverError> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        auth_user
            .select(user_id)
            .filter(user_email.eq(email_eq))
            .limit(limit)
            .order(user_id.asc())
            .load::<Uuid>(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn create(
        conn: &PgConnection,
        is_enabled: bool,
        name: &str,
        email: &str,
        password_hash: Option<&str>,
    ) -> Result<User, DriverError> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let now = Utc::now();
        let value = UserInsert {
            created_at: &now,
            updated_at: &now,
            user_id: Uuid::new_v4(),
            user_is_enabled: is_enabled,
            user_name: name,
            user_email: email,
            user_password_hash: password_hash,
        };
        diesel::insert_into(auth_user)
            .values(&value)
            .get_result::<User>(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn read_by_id(conn: &PgConnection, id: Uuid) -> Result<Option<User>, DriverError> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        auth_user
            .filter(user_id.eq(id))
            .get_result::<User>(conn)
            .map(Some)
            .or_else(|err| match err {
                DieselError::NotFound => Ok(None),
                _ => Err(DriverError::Diesel(err)),
            })
    }

    pub fn read_by_email(conn: &PgConnection, email: &str) -> Result<Option<User>, DriverError> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        auth_user
            .filter(user_email.eq(email))
            .get_result::<User>(conn)
            .map(Some)
            .or_else(|err| match err {
                DieselError::NotFound => Ok(None),
                _ => Err(DriverError::Diesel(err)),
            })
    }

    pub fn update_by_id(
        conn: &PgConnection,
        id: Uuid,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> Result<User, DriverError> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let now = Utc::now();
        let value = UserUpdate {
            updated_at: &now,
            user_is_enabled: is_enabled,
            user_name: name,
        };
        diesel::update(auth_user.filter(user_id.eq(id)))
            .set(&value)
            .get_result::<User>(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn update_email_by_id(
        conn: &PgConnection,
        id: Uuid,
        email: &str,
    ) -> Result<usize, DriverError> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let now = Utc::now();
        diesel::update(auth_user.filter(user_id.eq(id)))
            .set((updated_at.eq(now), user_email.eq(email)))
            .execute(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn update_password_by_id(
        conn: &PgConnection,
        id: Uuid,
        password_hash: &str,
    ) -> Result<usize, DriverError> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let now = Utc::now();
        diesel::update(auth_user.filter(user_id.eq(id)))
            .set((updated_at.eq(now), user_password_hash.eq(password_hash)))
            .execute(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn delete_by_id(conn: &PgConnection, id: Uuid) -> Result<usize, DriverError> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        diesel::delete(auth_user.filter(user_id.eq(id)))
            .execute(conn)
            .map_err(DriverError::Diesel)
    }
}
