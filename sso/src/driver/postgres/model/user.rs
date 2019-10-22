use crate::{
    driver::postgres::schema::sso_user, DriverResult, User, UserCreate, UserList, UserListQuery,
    UserRead, UserUpdate, UserUpdate2,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "sso_user"]
#[primary_key(id)]
pub struct ModelUser {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    id: Uuid,
    is_enabled: bool,
    name: String,
    email: String,
    locale: String,
    timezone: String,
    password_allow_reset: bool,
    password_require_update: bool,
    password_hash: Option<String>,
}

impl From<ModelUser> for User {
    fn from(user: ModelUser) -> Self {
        Self {
            created_at: user.created_at,
            updated_at: user.updated_at,
            id: user.id,
            is_enabled: user.is_enabled,
            name: user.name,
            email: user.email,
            locale: user.locale,
            timezone: user.timezone,
            password_allow_reset: user.password_allow_reset,
            password_require_update: user.password_require_update,
            password_hash: user.password_hash,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "sso_user"]
struct ModelUserInsert<'a> {
    created_at: &'a DateTime<Utc>,
    updated_at: &'a DateTime<Utc>,
    id: &'a Uuid,
    is_enabled: bool,
    name: &'a str,
    email: &'a str,
    locale: &'a str,
    timezone: &'a str,
    password_allow_reset: bool,
    password_require_update: bool,
    password_hash: Option<&'a str>,
}

impl<'a> ModelUserInsert<'a> {
    fn from_create(now: &'a DateTime<Utc>, id: &'a Uuid, create: &'a UserCreate) -> Self {
        Self {
            created_at: now,
            updated_at: now,
            id,
            is_enabled: create.is_enabled,
            name: &create.name,
            email: &create.email,
            locale: &create.locale,
            timezone: &create.timezone,
            password_allow_reset: create.password_allow_reset,
            password_require_update: create.password_require_update,
            password_hash: create.password_hash.as_ref().map(|x| &**x),
        }
    }
}

#[derive(AsChangeset)]
#[table_name = "sso_user"]
struct ModelUserUpdate<'a> {
    updated_at: &'a DateTime<Utc>,
    is_enabled: Option<bool>,
    name: Option<&'a str>,
    email: Option<&'a str>,
    locale: Option<&'a str>,
    timezone: Option<&'a str>,
    password_allow_reset: Option<bool>,
    password_require_update: Option<bool>,
    password_hash: Option<&'a str>,
}

impl<'a> ModelUserUpdate<'a> {
    fn from_update(now: &'a DateTime<Utc>, update: &'a UserUpdate) -> Self {
        Self {
            updated_at: now,
            is_enabled: update.is_enabled,
            name: update.name.as_ref().map(|x| &**x),
            email: None,
            locale: update.locale.as_ref().map(|x| &**x),
            timezone: update.timezone.as_ref().map(|x| &**x),
            password_allow_reset: update.password_allow_reset,
            password_require_update: update.password_require_update,
            password_hash: None,
        }
    }

    fn from_update2(now: &'a DateTime<Utc>, update: &'a UserUpdate2) -> Self {
        Self {
            updated_at: now,
            is_enabled: None,
            name: None,
            email: update.email.as_ref().map(|x| &**x),
            locale: None,
            timezone: None,
            password_allow_reset: None,
            password_require_update: None,
            password_hash: update.password_hash.as_ref().map(|x| &**x),
        }
    }
}

impl ModelUser {
    pub fn list(conn: &PgConnection, list: &UserList) -> DriverResult<Vec<User>> {
        use diesel::dsl::any;

        let mut query = sso_user::table.into_boxed();

        if let Some(id) = &list.filter.id {
            let id: Vec<Uuid> = id.iter().copied().collect();
            query = query.filter(sso_user::dsl::id.eq(any(id)));
        }
        if let Some(email) = &list.filter.email {
            let email: Vec<String> = email.to_vec();
            query = query.filter(sso_user::dsl::email.eq(any(email)));
        }

        match list.query {
            UserListQuery::Limit(limit) => query
                .filter(sso_user::dsl::id.gt(Uuid::nil()))
                .limit(*limit)
                .order(sso_user::dsl::id.asc())
                .load::<ModelUser>(conn)
                .map_err(Into::into)
                .map(|x| x.into_iter().map(|x| x.into()).collect()),
            UserListQuery::IdGt(gt, limit) => query
                .filter(sso_user::dsl::id.gt(gt))
                .limit(*limit)
                .order(sso_user::dsl::id.asc())
                .load::<ModelUser>(conn)
                .map_err(Into::into)
                .map(|x| x.into_iter().map(|x| x.into()).collect()),
            UserListQuery::IdLt(lt, limit) => query
                .filter(sso_user::dsl::id.lt(lt))
                .limit(*limit)
                .order(sso_user::dsl::id.desc())
                .load::<ModelUser>(conn)
                .map_err(Into::into)
                .map(|mut x| {
                    x.reverse();
                    x.into_iter().map(|x| x.into()).collect()
                }),
            // TODO(refactor): Implement this.
            UserListQuery::NameGe(_name_ge, _limit, _offset_id) => unimplemented!(),
            UserListQuery::NameLe(_name_le, _limit, _offset_id) => unimplemented!(),
        }
    }

    pub fn create(conn: &PgConnection, create: &UserCreate) -> DriverResult<User> {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let value = ModelUserInsert::from_create(&now, &id, create);
        diesel::insert_into(sso_user::table)
            .values(&value)
            .get_result::<ModelUser>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn read(conn: &PgConnection, read: &UserRead) -> DriverResult<User> {
        match read {
            UserRead::Id(id) => Self::read_id(conn, id),
            UserRead::Email(email) => Self::read_email(conn, email),
        }
    }

    pub fn read_opt(conn: &PgConnection, read: &UserRead) -> DriverResult<Option<User>> {
        match read {
            UserRead::Id(id) => Self::read_id_opt(conn, id),
            UserRead::Email(email) => Self::read_email_opt(conn, email),
        }
    }

    pub fn update(conn: &PgConnection, id: &Uuid, update: &UserUpdate) -> DriverResult<User> {
        let now = Utc::now();
        let value = ModelUserUpdate::from_update(&now, update);
        Self::update_inner(conn, id, &value)
    }

    pub fn update2(conn: &PgConnection, id: &Uuid, update: &UserUpdate2) -> DriverResult<User> {
        let now = Utc::now();
        let value = ModelUserUpdate::from_update2(&now, update);
        Self::update_inner(conn, id, &value)
    }

    pub fn delete(conn: &PgConnection, id: &Uuid) -> DriverResult<usize> {
        diesel::delete(sso_user::table.filter(sso_user::dsl::id.eq(id)))
            .execute(conn)
            .map_err(Into::into)
    }

    fn read_id(conn: &PgConnection, id: &Uuid) -> DriverResult<User> {
        Self::read_id_inner(conn, id)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn read_email(conn: &PgConnection, email: &str) -> DriverResult<User> {
        Self::read_email_inner(conn, email)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn read_id_opt(conn: &PgConnection, id: &Uuid) -> DriverResult<Option<User>> {
        Self::read_id_inner(conn, id)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn read_email_opt(conn: &PgConnection, email: &str) -> DriverResult<Option<User>> {
        Self::read_email_inner(conn, email)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn read_id_inner(conn: &PgConnection, id: &Uuid) -> QueryResult<ModelUser> {
        sso_user::table
            .filter(sso_user::dsl::id.eq(id))
            .get_result::<ModelUser>(conn)
    }

    fn read_email_inner(conn: &PgConnection, email: &str) -> QueryResult<ModelUser> {
        sso_user::table
            .filter(sso_user::dsl::email.eq(email))
            .get_result::<ModelUser>(conn)
    }

    fn update_inner(conn: &PgConnection, id: &Uuid, value: &ModelUserUpdate) -> DriverResult<User> {
        diesel::update(sso_user::table.filter(sso_user::dsl::id.eq(id)))
            .set(value)
            .get_result::<ModelUser>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }
}
