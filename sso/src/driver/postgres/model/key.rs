use crate::{
    driver::postgres::schema::sso_key, DriverResult, Key, KeyCount, KeyCreate, KeyList, KeyRead,
    KeyReadUserId, KeyUpdate,
};
use chrono::{DateTime, Utc};
use diesel::{dsl::sql, prelude::*, sql_types::BigInt, PgConnection};
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "sso_key"]
#[primary_key(key_id)]
pub struct ModelKey {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    key_id: Uuid,
    key_is_enabled: bool,
    key_is_revoked: bool,
    key_allow_key: bool,
    key_allow_token: bool,
    key_allow_totp: bool,
    key_name: String,
    key_value: String,
    service_id: Option<Uuid>,
    user_id: Option<Uuid>,
}

impl From<ModelKey> for Key {
    fn from(key: ModelKey) -> Self {
        Self {
            created_at: key.created_at,
            updated_at: key.updated_at,
            id: key.key_id,
            is_enabled: key.key_is_enabled,
            is_revoked: key.key_is_revoked,
            allow_key: key.key_allow_key,
            allow_token: key.key_allow_token,
            allow_totp: key.key_allow_totp,
            name: key.key_name,
            value: key.key_value,
            service_id: key.service_id,
            user_id: key.user_id,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "sso_key"]
struct ModelKeyInsert<'a> {
    created_at: &'a DateTime<Utc>,
    updated_at: &'a DateTime<Utc>,
    key_id: &'a Uuid,
    key_is_enabled: bool,
    key_is_revoked: bool,
    key_allow_key: bool,
    key_allow_token: bool,
    key_allow_totp: bool,
    key_name: &'a str,
    key_value: &'a str,
    service_id: Option<&'a Uuid>,
    user_id: Option<&'a Uuid>,
}

impl<'a> ModelKeyInsert<'a> {
    fn from_create(now: &'a DateTime<Utc>, id: &'a Uuid, create: &'a KeyCreate) -> Self {
        Self {
            created_at: now,
            updated_at: now,
            key_id: id,
            key_is_enabled: create.is_enabled,
            key_is_revoked: create.is_revoked,
            key_allow_key: create.allow_key,
            key_allow_token: create.allow_token,
            key_allow_totp: create.allow_totp,
            key_name: &create.name,
            key_value: &create.value,
            service_id: create.service_id.as_ref(),
            user_id: create.user_id.as_ref(),
        }
    }
}

#[derive(AsChangeset)]
#[table_name = "sso_key"]
struct ModelKeyUpdate<'a> {
    updated_at: &'a DateTime<Utc>,
    key_is_enabled: Option<bool>,
    key_is_revoked: Option<bool>,
    key_allow_key: Option<bool>,
    key_allow_token: Option<bool>,
    key_allow_totp: Option<bool>,
    key_name: Option<&'a str>,
}

impl<'a> ModelKeyUpdate<'a> {
    fn from_update(now: &'a DateTime<Utc>, update: &'a KeyUpdate) -> Self {
        Self {
            updated_at: now,
            key_is_enabled: update.is_enabled,
            key_is_revoked: update.is_revoked,
            key_allow_key: update.allow_key,
            key_allow_token: update.allow_token,
            key_allow_totp: update.allow_totp,
            key_name: update.name.as_ref().map(|x| &**x),
        }
    }
}

impl ModelKey {
    pub fn list(
        conn: &PgConnection,
        list: &KeyList,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Key>> {
        match list {
            KeyList::Limit(limit) => {
                let gt = Uuid::nil();
                Self::list_where_id_gt(conn, &gt, *limit, service_id_mask)
            }
            KeyList::IdGt(gt, limit) => Self::list_where_id_gt(conn, gt, *limit, service_id_mask),
            KeyList::IdLt(lt, limit) => Self::list_where_id_lt(conn, lt, *limit, service_id_mask),
        }
    }

    fn list_where_id_lt(
        conn: &PgConnection,
        lt: &Uuid,
        limit: i64,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Key>> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        match service_id_mask {
            Some(service_id_mask) => sso_key
                .filter(service_id.eq(service_id_mask).and(key_id.lt(lt)))
                .limit(limit)
                .order(key_id.desc())
                .load::<ModelKey>(conn),
            None => sso_key
                .filter(key_id.lt(lt))
                .limit(limit)
                .order(key_id.desc())
                .load::<ModelKey>(conn),
        }
        .map_err(Into::into)
        .map(|mut x| {
            x.reverse();
            x.into_iter().map(|x| x.into()).collect()
        })
    }

    fn list_where_id_gt(
        conn: &PgConnection,
        gt: &Uuid,
        limit: i64,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Key>> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        match service_id_mask {
            Some(service_id_mask) => sso_key
                .filter(service_id.eq(service_id_mask).and(key_id.gt(gt)))
                .limit(limit)
                .order(key_id.asc())
                .load::<ModelKey>(conn),
            None => sso_key
                .filter(key_id.gt(gt))
                .limit(limit)
                .order(key_id.asc())
                .load::<ModelKey>(conn),
        }
        .map_err(Into::into)
        .map(|x| x.into_iter().map(|x| x.into()).collect())
    }

    pub fn count(conn: &PgConnection, count: &KeyCount) -> DriverResult<usize> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        match count {
            KeyCount::AllowToken(count_service_id, count_user_id) => sso_key
                .select(sql::<BigInt>("count(*)"))
                .filter(
                    key_is_enabled
                        .eq(true)
                        .and(key_allow_token.eq(true))
                        .and(service_id.eq(count_service_id))
                        .and(user_id.eq(count_user_id)),
                )
                .get_result::<i64>(conn),
            KeyCount::AllowTotp(count_service_id, count_user_id) => sso_key
                .select(sql::<BigInt>("count(*)"))
                .filter(
                    key_is_enabled
                        .eq(true)
                        .and(key_allow_totp.eq(true))
                        .and(service_id.eq(count_service_id))
                        .and(user_id.eq(count_user_id)),
                )
                .get_result::<i64>(conn),
        }
        .map_err(Into::into)
        .map(|x| x as usize)
    }

    pub fn create(conn: &PgConnection, create: &KeyCreate) -> DriverResult<Key> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        let now = Utc::now();
        let id = Uuid::new_v4();
        let value = ModelKeyInsert::from_create(&now, &id, create);
        diesel::insert_into(sso_key)
            .values(&value)
            .get_result::<ModelKey>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn read_opt(conn: &PgConnection, read: &KeyRead) -> DriverResult<Option<Key>> {
        match read {
            KeyRead::Id(id) => Self::read_by_id(conn, id),
            KeyRead::UserId(r) => Self::read_by_user_id(conn, r),
            KeyRead::RootValue(value) => Self::read_by_root_value(conn, value),
            KeyRead::ServiceValue(value) => Self::read_by_service_value(conn, value),
            KeyRead::UserValue(service_id, value) => {
                Self::read_by_user_value(conn, service_id, value)
            }
        }
    }

    fn read_by_id(conn: &PgConnection, id: &Uuid) -> DriverResult<Option<Key>> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        sso_key
            .filter(key_id.eq(id))
            .get_result::<ModelKey>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn read_by_user_id(conn: &PgConnection, read: &KeyReadUserId) -> DriverResult<Option<Key>> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        sso_key
            .filter(
                user_id
                    .eq(read.user_id)
                    .and(service_id.eq(read.service_id))
                    .and(key_is_enabled.eq(read.is_enabled))
                    .and(key_is_revoked.eq(read.is_revoked)),
            )
            .order(created_at.asc())
            .get_result::<ModelKey>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn read_by_root_value(conn: &PgConnection, value: &str) -> DriverResult<Option<Key>> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        sso_key
            .filter(
                key_value
                    .eq(value)
                    .and(service_id.is_null())
                    .and(user_id.is_null()),
            )
            .get_result::<ModelKey>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn read_by_service_value(conn: &PgConnection, value: &str) -> DriverResult<Option<Key>> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        sso_key
            .filter(
                key_value
                    .eq(value)
                    .and(service_id.is_not_null())
                    .and(user_id.is_null()),
            )
            .get_result::<ModelKey>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn read_by_user_value(
        conn: &PgConnection,
        key_service_id: &Uuid,
        value: &str,
    ) -> DriverResult<Option<Key>> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        sso_key
            .filter(
                key_value
                    .eq(value)
                    .and(service_id.eq(key_service_id).and(user_id.is_not_null())),
            )
            .get_result::<ModelKey>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    pub fn update(conn: &PgConnection, id: &Uuid, update: &KeyUpdate) -> DriverResult<Key> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        let now = chrono::Utc::now();
        let value = ModelKeyUpdate::from_update(&now, update);
        diesel::update(sso_key.filter(key_id.eq(id)))
            .set(&value)
            .get_result::<ModelKey>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn update_many(
        conn: &PgConnection,
        key_user_id: &Uuid,
        update: &KeyUpdate,
    ) -> DriverResult<usize> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        let now = chrono::Utc::now();
        let value = ModelKeyUpdate::from_update(&now, update);
        diesel::update(sso_key.filter(user_id.eq(key_user_id)))
            .set(&value)
            .execute(conn)
            .map_err(Into::into)
    }

    pub fn delete(conn: &PgConnection, id: &Uuid) -> DriverResult<usize> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        diesel::delete(sso_key.filter(key_id.eq(id)))
            .execute(conn)
            .map_err(Into::into)
    }
}
