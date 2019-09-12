use crate::{driver::postgres::schema::auth_user, DriverResult, User, UserCreate, UserUpdate};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_user"]
#[primary_key(user_id)]
pub struct ModelUser {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    user_id: Uuid,
    user_is_enabled: bool,
    user_name: String,
    user_email: String,
    user_password_hash: Option<String>,
}

impl From<ModelUser> for User {
    fn from(user: ModelUser) -> Self {
        Self {
            created_at: user.created_at,
            updated_at: user.updated_at,
            id: user.user_id,
            is_enabled: user.user_is_enabled,
            name: user.user_name,
            email: user.user_email,
            password_hash: user.user_password_hash,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "auth_user"]
struct ModelUserInsert<'a> {
    created_at: &'a DateTime<Utc>,
    updated_at: &'a DateTime<Utc>,
    user_id: &'a Uuid,
    user_is_enabled: bool,
    user_name: &'a str,
    user_email: &'a str,
    user_password_hash: Option<&'a str>,
}

impl<'a> ModelUserInsert<'a> {
    fn from_create(now: &'a DateTime<Utc>, id: &'a Uuid, create: &'a UserCreate) -> Self {
        Self {
            created_at: now,
            updated_at: now,
            user_id: id,
            user_is_enabled: create.is_enabled,
            user_name: create.name,
            user_email: create.email,
            user_password_hash: create.password_hash,
        }
    }
}

#[derive(AsChangeset)]
#[table_name = "auth_user"]
struct ModelUserUpdate<'a> {
    updated_at: &'a DateTime<Utc>,
    user_is_enabled: Option<bool>,
    user_name: Option<&'a str>,
}

impl<'a> ModelUserUpdate<'a> {
    fn from_update(now: &'a DateTime<Utc>, update: &'a UserUpdate) -> Self {
        Self {
            updated_at: now,
            user_is_enabled: update.is_enabled,
            user_name: update.name,
        }
    }
}

impl ModelUser {
    pub fn list_where_id_lt(conn: &PgConnection, lt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        auth_user
            .select(user_id)
            .filter(user_id.lt(lt))
            .limit(limit)
            .order(user_id.desc())
            .load::<Uuid>(conn)
            .map_err(Into::into)
            .map(|mut v| {
                v.reverse();
                v
            })
    }

    pub fn list_where_id_gt(conn: &PgConnection, gt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        auth_user
            .select(user_id)
            .filter(user_id.gt(gt))
            .limit(limit)
            .order(user_id.asc())
            .load::<Uuid>(conn)
            .map_err(Into::into)
    }

    pub fn list_where_email_eq(
        conn: &PgConnection,
        email_eq: &str,
        limit: i64,
    ) -> DriverResult<Vec<Uuid>> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        auth_user
            .select(user_id)
            .filter(user_email.eq(email_eq))
            .limit(limit)
            .order(user_id.asc())
            .load::<Uuid>(conn)
            .map_err(Into::into)
    }

    pub fn create(conn: &PgConnection, create: &UserCreate) -> DriverResult<User> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let now = Utc::now();
        let id = Uuid::new_v4();
        let value = ModelUserInsert::from_create(&now, &id, create);
        diesel::insert_into(auth_user)
            .values(&value)
            .get_result::<ModelUser>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn read_by_id(conn: &PgConnection, id: Uuid) -> DriverResult<Option<User>> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        auth_user
            .filter(user_id.eq(id))
            .get_result::<ModelUser>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    pub fn read_by_email(conn: &PgConnection, email: &str) -> DriverResult<Option<User>> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        auth_user
            .filter(user_email.eq(email))
            .get_result::<ModelUser>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    pub fn update_by_id(conn: &PgConnection, id: Uuid, update: &UserUpdate) -> DriverResult<User> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let now = Utc::now();
        let value = ModelUserUpdate::from_update(&now, update);
        diesel::update(auth_user.filter(user_id.eq(id)))
            .set(&value)
            .get_result::<ModelUser>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn update_email_by_id(conn: &PgConnection, id: Uuid, email: &str) -> DriverResult<usize> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let now = Utc::now();
        diesel::update(auth_user.filter(user_id.eq(id)))
            .set((updated_at.eq(now), user_email.eq(email)))
            .execute(conn)
            .map_err(Into::into)
    }

    pub fn update_password_by_id(
        conn: &PgConnection,
        id: Uuid,
        password_hash: &str,
    ) -> DriverResult<usize> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let now = Utc::now();
        diesel::update(auth_user.filter(user_id.eq(id)))
            .set((updated_at.eq(now), user_password_hash.eq(password_hash)))
            .execute(conn)
            .map_err(Into::into)
    }

    pub fn delete_by_id(conn: &PgConnection, id: Uuid) -> DriverResult<usize> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        diesel::delete(auth_user.filter(user_id.eq(id)))
            .execute(conn)
            .map_err(Into::into)
    }
}
