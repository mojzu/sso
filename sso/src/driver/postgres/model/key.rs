use crate::{
    driver::postgres::schema::sso_key, DriverResult, Key, KeyCount, KeyCreate, KeyFilter, KeyList,
    KeyRead, KeyReadUserId, KeyReadUserValue, KeyType, KeyUpdate,
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
    key_type: String,
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
            type_: KeyType::from_string(&key.key_type).unwrap(),
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
    key_type: String,
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
            key_type: create.type_.to_string().unwrap(),
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
    key_name: Option<&'a str>,
}

impl<'a> ModelKeyUpdate<'a> {
    fn from_update(now: &'a DateTime<Utc>, update: &'a KeyUpdate) -> Self {
        Self {
            updated_at: now,
            key_is_enabled: update.is_enabled,
            key_is_revoked: update.is_revoked,
            key_name: update.name.as_ref().map(|x| &**x),
        }
    }
}

impl ModelKey {
    pub fn list(
        conn: &PgConnection,
        list: &KeyList,
        filter: &KeyFilter,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Key>> {
        use diesel::dsl::any;

        let mut query = sso_key::table.into_boxed();

        if let Some(is_enabled) = filter.is_enabled {
            query = query.filter(sso_key::dsl::key_is_enabled.eq(is_enabled));
        }
        if let Some(is_revoked) = filter.is_revoked {
            query = query.filter(sso_key::dsl::key_is_revoked.eq(is_revoked));
        }
        if let Some(type_) = &filter.type_ {
            let type_: Vec<String> = type_.iter().map(|x| x.to_string().unwrap()).collect();
            query = query.filter(sso_key::dsl::key_type.eq(any(type_)));
        }
        if let Some(service_id) = &filter.service_id {
            let service_id: Vec<Uuid> = service_id.iter().copied().collect();
            query = query.filter(sso_key::dsl::service_id.eq(any(service_id)));
        }
        if let Some(user_id) = &filter.user_id {
            let user_id: Vec<Uuid> = user_id.iter().copied().collect();
            query = query.filter(sso_key::dsl::user_id.eq(any(user_id)));
        }
        if let Some(service_id_mask) = service_id_mask {
            query = query.filter(sso_key::dsl::service_id.eq(service_id_mask.clone()));
        }

        match list {
            KeyList::Limit(limit) => query
                .filter(sso_key::dsl::key_id.gt(Uuid::nil()))
                .limit(*limit)
                .order(sso_key::dsl::key_id.asc())
                .load::<ModelKey>(conn)
                .map_err(Into::into)
                .map(|x| x.into_iter().map(|x| x.into()).collect()),
            KeyList::IdGt(gt, limit) => query
                .filter(sso_key::dsl::key_id.gt(gt))
                .limit(*limit)
                .order(sso_key::dsl::key_id.asc())
                .load::<ModelKey>(conn)
                .map_err(Into::into)
                .map(|x| x.into_iter().map(|x| x.into()).collect()),
            KeyList::IdLt(lt, limit) => query
                .filter(sso_key::dsl::key_id.lt(lt))
                .limit(*limit)
                .order(sso_key::dsl::key_id.desc())
                .load::<ModelKey>(conn)
                .map_err(Into::into)
                .map(|mut x| {
                    x.reverse();
                    x.into_iter().map(|x| x.into()).collect()
                }),
        }
    }

    pub fn count(conn: &PgConnection, count: &KeyCount) -> DriverResult<usize> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        match count {
            KeyCount::Token(count_service_id, count_user_id) => sso_key
                .select(sql::<BigInt>("count(*)"))
                .filter(
                    key_is_enabled
                        .eq(true)
                        .and(key_type.eq("Token"))
                        .and(service_id.eq(count_service_id))
                        .and(user_id.eq(count_user_id)),
                )
                .get_result::<i64>(conn),
            KeyCount::Totp(count_service_id, count_user_id) => sso_key
                .select(sql::<BigInt>("count(*)"))
                .filter(
                    key_is_enabled
                        .eq(true)
                        .and(key_type.eq("Totp"))
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
            KeyRead::RootValue(value) => Self::read_by_root_value(conn, value),
            KeyRead::ServiceValue(value) => Self::read_by_service_value(conn, value),
            KeyRead::UserId(r) => Self::read_by_user_id(conn, r),
            KeyRead::UserValue(r) => Self::read_by_user_value(conn, r),
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

    fn read_by_user_id(conn: &PgConnection, read: &KeyReadUserId) -> DriverResult<Option<Key>> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        let type_ = read.type_.to_string().unwrap();
        sso_key
            .filter(
                user_id
                    .eq(read.user_id)
                    .and(service_id.eq(read.service_id))
                    .and(key_is_enabled.eq(read.is_enabled))
                    .and(key_is_revoked.eq(read.is_revoked))
                    .and(key_type.eq(type_)),
            )
            .order(created_at.asc())
            .get_result::<ModelKey>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn read_by_user_value(
        conn: &PgConnection,
        read: &KeyReadUserValue,
    ) -> DriverResult<Option<Key>> {
        use crate::driver::postgres::schema::sso_key::dsl::*;

        let type_ = read.type_.to_string().unwrap();
        sso_key
            .filter(
                key_value
                    .eq(&read.value)
                    .and(service_id.eq(read.service_id))
                    .and(user_id.is_not_null())
                    .and(key_is_enabled.eq(read.is_enabled))
                    .and(key_is_revoked.eq(read.is_revoked))
                    .and(key_type.eq(type_)),
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
