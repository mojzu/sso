use crate::{
    driver::postgres::{
        model::{ModelService, ModelUser},
        schema::sso_key,
    },
    DriverError, DriverResult, Key, KeyCount, KeyCreate, KeyList, KeyListQuery, KeyRead,
    KeyReadUserId, KeyReadUserValue, KeyType, KeyUpdate, KeyWithValue, ServiceRead, UserRead,
};
use chrono::{DateTime, Utc};
use diesel::{dsl::sql, prelude::*, sql_types::BigInt, PgConnection};
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "sso_key"]
#[primary_key(id)]
pub struct ModelKey {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    id: Uuid,
    is_enabled: bool,
    is_revoked: bool,
    type_: String,
    name: String,
    value: String,
    service_id: Option<Uuid>,
    user_id: Option<Uuid>,
}

impl From<ModelKey> for Key {
    fn from(key: ModelKey) -> Self {
        Self {
            created_at: key.created_at,
            updated_at: key.updated_at,
            id: key.id,
            is_enabled: key.is_enabled,
            is_revoked: key.is_revoked,
            type_: KeyType::from_string(&key.type_).unwrap(),
            name: key.name,
            service_id: key.service_id,
            user_id: key.user_id,
        }
    }
}

impl From<ModelKey> for KeyWithValue {
    fn from(key: ModelKey) -> Self {
        Self {
            created_at: key.created_at,
            updated_at: key.updated_at,
            id: key.id,
            is_enabled: key.is_enabled,
            is_revoked: key.is_revoked,
            type_: KeyType::from_string(&key.type_).unwrap(),
            name: key.name,
            value: key.value,
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
    id: &'a Uuid,
    is_enabled: bool,
    is_revoked: bool,
    type_: String,
    name: &'a str,
    value: &'a str,
    service_id: Option<&'a Uuid>,
    user_id: Option<&'a Uuid>,
}

#[derive(AsChangeset)]
#[table_name = "sso_key"]
struct ModelKeyUpdate<'a> {
    updated_at: &'a DateTime<Utc>,
    is_enabled: Option<bool>,
    is_revoked: Option<bool>,
    name: Option<&'a str>,
}

impl<'a> ModelKeyUpdate<'a> {
    fn from_update(now: &'a DateTime<Utc>, update: &'a KeyUpdate) -> Self {
        Self {
            updated_at: now,
            is_enabled: update.is_enabled,
            is_revoked: update.is_revoked,
            name: update.name.as_ref().map(|x| &**x),
        }
    }
}

impl ModelKey {
    pub fn list(
        conn: &PgConnection,
        list: &KeyList,
        service_id: Option<Uuid>,
    ) -> DriverResult<Vec<Key>> {
        use diesel::dsl::any;

        let mut query = sso_key::table.into_boxed();

        if let Some(id) = &list.filter.id {
            let id: Vec<Uuid> = id.iter().copied().collect();
            query = query.filter(sso_key::dsl::id.eq(any(id)));
        }
        if let Some(is_enabled) = list.filter.is_enabled {
            query = query.filter(sso_key::dsl::is_enabled.eq(is_enabled));
        }
        if let Some(is_revoked) = list.filter.is_revoked {
            query = query.filter(sso_key::dsl::is_revoked.eq(is_revoked));
        }
        if let Some(type_) = &list.filter.type_ {
            let type_: Vec<String> = type_.iter().map(|x| x.to_string().unwrap()).collect();
            query = query.filter(sso_key::dsl::type_.eq(any(type_)));
        }
        if let Some(service_id) = &list.filter.service_id {
            let service_id: Vec<Uuid> = service_id.iter().copied().collect();
            query = query.filter(sso_key::dsl::service_id.eq(any(service_id)));
        }
        if let Some(user_id) = &list.filter.user_id {
            let user_id: Vec<Uuid> = user_id.iter().copied().collect();
            query = query.filter(sso_key::dsl::user_id.eq(any(user_id)));
        }
        if let Some(service_id_mask) = service_id {
            query = query.filter(sso_key::dsl::service_id.eq(service_id_mask));
        }

        match list.query {
            KeyListQuery::Limit => query
                .filter(sso_key::dsl::id.gt(Uuid::nil()))
                .limit(list.filter.limit)
                .order(sso_key::dsl::id.asc())
                .load::<ModelKey>(conn)
                .map_err(Into::into)
                .map(|x| x.into_iter().map(|x| x.into()).collect()),
            KeyListQuery::IdGt(gt) => query
                .filter(sso_key::dsl::id.gt(gt))
                .limit(list.filter.limit)
                .order(sso_key::dsl::id.asc())
                .load::<ModelKey>(conn)
                .map_err(Into::into)
                .map(|x| x.into_iter().map(|x| x.into()).collect()),
            KeyListQuery::IdLt(lt) => query
                .filter(sso_key::dsl::id.lt(lt))
                .limit(list.filter.limit)
                .order(sso_key::dsl::id.desc())
                .load::<ModelKey>(conn)
                .map_err(Into::into)
                .map(|mut x| {
                    x.reverse();
                    x.into_iter().map(|x| x.into()).collect()
                }),
        }
    }

    pub fn count(conn: &PgConnection, count: &KeyCount) -> DriverResult<usize> {
        match count {
            KeyCount::Token(service_id, user_id) => Self::count_token(conn, service_id, user_id),
            KeyCount::Totp(service_id, user_id) => Self::count_totp(conn, service_id, user_id),
        }
        .map(|x| x as usize)
    }

    pub fn create(conn: &PgConnection, create: &KeyCreate) -> DriverResult<KeyWithValue> {
        if create.is_enabled {
            if create.type_ == KeyType::Token {
                let count = Self::count_token(
                    conn,
                    create.service_id.as_ref().unwrap(),
                    create.user_id.as_ref().unwrap(),
                )?;
                if count != 0 {
                    return Err(DriverError::KeyUserTokenConstraint);
                }
            }
            if create.type_ == KeyType::Totp {
                let count = Self::count_totp(
                    conn,
                    create.service_id.as_ref().unwrap(),
                    create.user_id.as_ref().unwrap(),
                )?;
                if count != 0 {
                    return Err(DriverError::KeyUserTotpConstraint);
                }
            }
        }
        if let Some(service_id) = &create.service_id {
            ModelService::read(conn, &ServiceRead::new(*service_id), None)?
                .ok_or_else(|| DriverError::ServiceNotFound)?;
        }
        if let Some(user_id) = &create.user_id {
            ModelUser::read(conn, &UserRead::Id(*user_id))?
                .ok_or_else(|| DriverError::UserNotFound)?;
        }

        let now = Utc::now();
        let id = Uuid::new_v4();
        let value = ModelKeyInsert {
            created_at: &now,
            updated_at: &now,
            id: &id,
            is_enabled: create.is_enabled,
            is_revoked: create.is_revoked,
            type_: create.type_.to_string().unwrap(),
            name: &create.name,
            value: &create.value,
            service_id: create.service_id.as_ref(),
            user_id: create.user_id.as_ref(),
        };
        diesel::insert_into(sso_key::table)
            .values(&value)
            .get_result::<ModelKey>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn read(
        conn: &PgConnection,
        read: &KeyRead,
        service_id: Option<Uuid>,
    ) -> DriverResult<Option<KeyWithValue>> {
        match read {
            KeyRead::IdUser(id, user_id) => Self::read_by_id(conn, id, &service_id, user_id),
            KeyRead::RootValue(value) => Self::read_by_root_value(conn, value),
            KeyRead::ServiceValue(value) => Self::read_by_service_value(conn, value),
            KeyRead::UserId(r) => Self::read_by_user_id(conn, r),
            KeyRead::UserValue(r) => Self::read_by_user_value(conn, r),
        }
    }

    pub fn update(conn: &PgConnection, update: &KeyUpdate) -> DriverResult<Key> {
        let now = chrono::Utc::now();
        let value = ModelKeyUpdate::from_update(&now, update);
        diesel::update(sso_key::table.filter(sso_key::dsl::id.eq(update.id)))
            .set(&value)
            .get_result::<ModelKey>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn update_many(
        conn: &PgConnection,
        user_id: &Uuid,
        update: &KeyUpdate,
    ) -> DriverResult<usize> {
        let now = chrono::Utc::now();
        let value = ModelKeyUpdate::from_update(&now, update);
        diesel::update(sso_key::table.filter(sso_key::dsl::user_id.eq(user_id)))
            .set(&value)
            .execute(conn)
            .map_err(Into::into)
    }

    pub fn delete(conn: &PgConnection, id: &Uuid) -> DriverResult<usize> {
        diesel::delete(sso_key::table.filter(sso_key::dsl::id.eq(id)))
            .execute(conn)
            .map_err(Into::into)
    }

    fn count_token(conn: &PgConnection, service_id: &Uuid, user_id: &Uuid) -> DriverResult<i64> {
        sso_key::table
            .select(sql::<BigInt>("count(*)"))
            .filter(
                sso_key::dsl::is_enabled
                    .eq(true)
                    .and(sso_key::dsl::type_.eq("Token"))
                    .and(sso_key::dsl::service_id.eq(service_id))
                    .and(sso_key::dsl::user_id.eq(user_id)),
            )
            .get_result::<i64>(conn)
            .map_err(Into::into)
    }

    fn count_totp(conn: &PgConnection, service_id: &Uuid, user_id: &Uuid) -> DriverResult<i64> {
        sso_key::table
            .select(sql::<BigInt>("count(*)"))
            .filter(
                sso_key::dsl::is_enabled
                    .eq(true)
                    .and(sso_key::dsl::type_.eq("Totp"))
                    .and(sso_key::dsl::service_id.eq(service_id))
                    .and(sso_key::dsl::user_id.eq(user_id)),
            )
            .get_result::<i64>(conn)
            .map_err(Into::into)
    }

    fn read_by_id(
        conn: &PgConnection,
        id: &Uuid,
        service_id_mask: &Option<Uuid>,
        user_id_mask: &Option<Uuid>,
    ) -> DriverResult<Option<KeyWithValue>> {
        let mut query = sso_key::table.into_boxed();

        if let Some(service_id) = service_id_mask {
            query = query.filter(sso_key::dsl::service_id.eq(service_id));
        }
        if let Some(user_id) = user_id_mask {
            query = query.filter(sso_key::dsl::user_id.eq(user_id));
        }

        query
            .filter(sso_key::dsl::id.eq(id))
            .get_result::<ModelKey>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn read_by_root_value(conn: &PgConnection, value: &str) -> DriverResult<Option<KeyWithValue>> {
        sso_key::table
            .filter(
                sso_key::dsl::value
                    .eq(value)
                    .and(sso_key::dsl::service_id.is_null())
                    .and(sso_key::dsl::user_id.is_null()),
            )
            .get_result::<ModelKey>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn read_by_service_value(
        conn: &PgConnection,
        value: &str,
    ) -> DriverResult<Option<KeyWithValue>> {
        sso_key::table
            .filter(
                sso_key::dsl::value
                    .eq(value)
                    .and(sso_key::dsl::service_id.is_not_null())
                    .and(sso_key::dsl::user_id.is_null()),
            )
            .get_result::<ModelKey>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn read_by_user_id(
        conn: &PgConnection,
        read: &KeyReadUserId,
    ) -> DriverResult<Option<KeyWithValue>> {
        let type_ = read.type_.to_string().unwrap();
        sso_key::table
            .filter(
                sso_key::dsl::user_id
                    .eq(read.user_id)
                    .and(sso_key::dsl::service_id.eq(read.service_id))
                    .and(sso_key::dsl::is_enabled.eq(read.is_enabled))
                    .and(sso_key::dsl::is_revoked.eq(read.is_revoked))
                    .and(sso_key::dsl::type_.eq(type_)),
            )
            .order(sso_key::dsl::created_at.asc())
            .get_result::<ModelKey>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn read_by_user_value(
        conn: &PgConnection,
        read: &KeyReadUserValue,
    ) -> DriverResult<Option<KeyWithValue>> {
        let type_ = read.type_.to_string().unwrap();
        sso_key::table
            .filter(
                sso_key::dsl::value
                    .eq(&read.value)
                    .and(sso_key::dsl::service_id.eq(read.service_id))
                    .and(sso_key::dsl::user_id.is_not_null())
                    .and(sso_key::dsl::is_enabled.eq(read.is_enabled))
                    .and(sso_key::dsl::is_revoked.eq(read.is_revoked))
                    .and(sso_key::dsl::type_.eq(type_)),
            )
            .get_result::<ModelKey>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }
}
