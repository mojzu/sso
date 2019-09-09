use crate::{driver::postgres::schema::auth_key, DriverError};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_key"]
#[primary_key(key_id)]
pub struct Key {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub key_id: Uuid,
    pub key_is_enabled: bool,
    pub key_is_revoked: bool,
    pub key_name: String,
    pub key_value: String,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Insertable)]
#[table_name = "auth_key"]
pub struct KeyInsert<'a> {
    pub created_at: &'a DateTime<Utc>,
    pub updated_at: &'a DateTime<Utc>,
    pub key_id: Uuid,
    pub key_is_enabled: bool,
    pub key_is_revoked: bool,
    pub key_name: &'a str,
    pub key_value: &'a str,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

#[derive(AsChangeset)]
#[table_name = "auth_key"]
pub struct KeyUpdate<'a> {
    pub updated_at: &'a DateTime<Utc>,
    pub key_is_enabled: Option<bool>,
    pub key_is_revoked: Option<bool>,
    pub key_name: Option<&'a str>,
}

impl Key {
    pub fn list_where_id_lt(
        conn: &PgConnection,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        match service_id_mask {
            Some(service_id_mask) => auth_key
                .select(key_id)
                .filter(service_id.eq(service_id_mask).and(key_id.lt(lt)))
                .limit(limit)
                .order(key_id.desc())
                .load::<Uuid>(conn),
            None => auth_key
                .select(key_id)
                .filter(key_id.lt(lt))
                .limit(limit)
                .order(key_id.desc())
                .load::<Uuid>(conn),
        }
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
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        match service_id_mask {
            Some(service_id_mask) => auth_key
                .select(key_id)
                .filter(service_id.eq(service_id_mask).and(key_id.gt(gt)))
                .limit(limit)
                .order(key_id.asc())
                .load::<Uuid>(conn),
            None => auth_key
                .select(key_id)
                .filter(key_id.gt(gt))
                .limit(limit)
                .order(key_id.asc())
                .load::<Uuid>(conn),
        }
        .map_err(DriverError::Diesel)
    }

    pub fn create(
        conn: &PgConnection,
        is_enabled: bool,
        is_revoked: bool,
        name: &str,
        value: &str,
        key_service_id: Option<Uuid>,
        key_user_id: Option<Uuid>,
    ) -> Result<Key, DriverError> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let now = Utc::now();
        let value = KeyInsert {
            created_at: &now,
            updated_at: &now,
            key_id: Uuid::new_v4(),
            key_is_enabled: is_enabled,
            key_is_revoked: is_revoked,
            key_name: name,
            key_value: value,
            service_id: key_service_id,
            user_id: key_user_id,
        };
        diesel::insert_into(auth_key)
            .values(&value)
            .get_result::<Key>(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn read_by_id(conn: &PgConnection, id: Uuid) -> Result<Option<Key>, DriverError> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        auth_key
            .filter(key_id.eq(id))
            .get_result::<Key>(conn)
            .map(Some)
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(DriverError::Diesel(err)),
            })
    }

    pub fn read_by_user_id(
        conn: &PgConnection,
        key_service_id: Uuid,
        key_user_id: Uuid,
    ) -> Result<Option<Key>, DriverError> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        auth_key
            .filter(
                user_id
                    .eq(key_user_id)
                    .and(service_id.eq(key_service_id))
                    // TODO(refactor): Pass these as arguments.
                    .and(key_is_enabled.eq(true))
                    .and(key_is_revoked.eq(false)),
            )
            .order(created_at.asc())
            .get_result::<Key>(conn)
            .map(Some)
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(DriverError::Diesel(err)),
            })
    }

    pub fn read_by_root_value(
        conn: &PgConnection,
        value: &str,
    ) -> Result<Option<Key>, DriverError> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        auth_key
            .filter(
                key_value
                    .eq(value)
                    .and(service_id.is_null())
                    .and(user_id.is_null()),
            )
            .get_result::<Key>(conn)
            .map(Some)
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(DriverError::Diesel(err)),
            })
    }

    pub fn read_by_service_value(
        conn: &PgConnection,
        value: &str,
    ) -> Result<Option<Key>, DriverError> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        auth_key
            .filter(
                key_value
                    .eq(value)
                    .and(service_id.is_not_null())
                    .and(user_id.is_null()),
            )
            .get_result::<Key>(conn)
            .map(Some)
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(DriverError::Diesel(err)),
            })
    }

    pub fn read_by_user_value(
        conn: &PgConnection,
        key_service_id: Uuid,
        value: &str,
    ) -> Result<Option<Key>, DriverError> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        auth_key
            .filter(
                key_value
                    .eq(value)
                    .and(service_id.eq(key_service_id).and(user_id.is_not_null())),
            )
            .get_result::<Key>(conn)
            .map(Some)
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(DriverError::Diesel(err)),
            })
    }

    pub fn update_by_id(
        conn: &PgConnection,
        id: Uuid,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> Result<Key, DriverError> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let now = chrono::Utc::now();
        let value = KeyUpdate {
            updated_at: &now,
            key_is_enabled: is_enabled,
            key_is_revoked: is_revoked,
            key_name: name,
        };
        diesel::update(auth_key.filter(key_id.eq(id)))
            .set(&value)
            .get_result::<Key>(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn update_many_by_user_id(
        conn: &PgConnection,
        key_user_id: Uuid,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> Result<usize, DriverError> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let now = chrono::Utc::now();
        let value = KeyUpdate {
            updated_at: &now,
            key_is_enabled: is_enabled,
            key_is_revoked: is_revoked,
            key_name: name,
        };
        diesel::update(auth_key.filter(user_id.eq(key_user_id)))
            .set(&value)
            .execute(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn delete_by_id(conn: &PgConnection, id: Uuid) -> Result<usize, DriverError> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        diesel::delete(auth_key.filter(key_id.eq(id)))
            .execute(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn delete_root(conn: &PgConnection) -> Result<usize, DriverError> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        diesel::delete(auth_key.filter(service_id.is_null().and(user_id.is_null())))
            .execute(conn)
            .map_err(DriverError::Diesel)
    }
}
