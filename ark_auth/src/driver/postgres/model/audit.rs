use crate::{
    core::AuditCreate,
    driver::{postgres::schema::auth_audit, DriverError},
};
use chrono::{DateTime, Utc};
use diesel::{dsl::sql, prelude::*, sql_types::BigInt, PgConnection};
use serde_json::Value;
use std::convert::TryInto;
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_audit"]
#[primary_key(audit_id)]
pub struct Audit {
    pub created_at: DateTime<Utc>,
    pub audit_id: Uuid,
    pub audit_user_agent: String,
    pub audit_remote: String,
    pub audit_forwarded: Option<String>,
    pub audit_path: String,
    pub audit_data: Value,
    pub key_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub user_key_id: Option<Uuid>,
}

#[derive(Debug, Insertable)]
#[table_name = "auth_audit"]
pub struct AuditInsert<'a> {
    pub created_at: &'a DateTime<Utc>,
    pub audit_id: Uuid,
    pub audit_user_agent: &'a str,
    pub audit_remote: &'a str,
    pub audit_forwarded: Option<&'a str>,
    pub audit_path: &'a str,
    pub audit_data: &'a Value,
    pub key_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub user_key_id: Option<Uuid>,
}

impl Audit {
    pub fn list_where_id_lt(
        conn: &PgConnection,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        use crate::driver::postgres::schema::auth_audit::dsl::*;

        match service_id_mask {
            Some(service_id_mask) => auth_audit
                .select(audit_id)
                .filter(service_id.eq(service_id_mask).and(audit_id.lt(lt)))
                .limit(limit)
                .order(audit_id.desc())
                .load::<Uuid>(conn),
            None => auth_audit
                .select(audit_id)
                .filter(audit_id.lt(lt))
                .limit(limit)
                .order(audit_id.desc())
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
        use crate::driver::postgres::schema::auth_audit::dsl::*;

        match service_id_mask {
            Some(service_id_mask) => auth_audit
                .select(audit_id)
                .filter(service_id.eq(service_id_mask).and(audit_id.gt(gt)))
                .limit(limit)
                .order(audit_id.asc())
                .load::<Uuid>(conn),
            None => auth_audit
                .select(audit_id)
                .filter(audit_id.gt(gt))
                .limit(limit)
                .order(audit_id.asc())
                .load::<Uuid>(conn),
        }
        .map_err(DriverError::Diesel)
    }

    pub fn list_where_id_gt_and_lt(
        conn: &PgConnection,
        gt: Uuid,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        use crate::driver::postgres::schema::auth_audit::dsl::*;

        match service_id_mask {
            Some(service_id_mask) => auth_audit
                .select(audit_id)
                .filter(
                    service_id
                        .eq(service_id_mask)
                        .and(audit_id.gt(gt))
                        .and(audit_id.lt(lt)),
                )
                .limit(limit)
                .order(audit_id.asc())
                .load::<Uuid>(conn),
            None => auth_audit
                .select(audit_id)
                .filter(audit_id.gt(gt).and(audit_id.lt(lt)))
                .limit(limit)
                .order(audit_id.asc())
                .load::<Uuid>(conn),
        }
        .map_err(DriverError::Diesel)
    }

    pub fn list_where_created_lte(
        conn: &PgConnection,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        Audit::list_where_created_lte_inner(conn, created_lte, limit, offset, service_id_mask)
            .and_then(|res| {
                if let Some(offset_id) = offset_id {
                    for (i, id) in res.iter().enumerate() {
                        if id == &offset_id {
                            let offset: i64 = (i + 1).try_into().unwrap();
                            return Audit::list_where_created_lte_inner(
                                conn,
                                created_lte,
                                limit,
                                offset,
                                service_id_mask,
                            );
                        }
                    }
                }
                Ok(res)
            })
            .map(|mut v| {
                v.reverse();
                v
            })
    }

    pub fn list_where_created_gte(
        conn: &PgConnection,
        created_gte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        Audit::list_where_created_gte_inner(conn, created_gte, limit, offset, service_id_mask)
            .and_then(|res| {
                if let Some(offset_id) = offset_id {
                    for (i, id) in res.iter().enumerate() {
                        if id == &offset_id {
                            let offset: i64 = (i + 1).try_into().unwrap();
                            return Audit::list_where_created_gte_inner(
                                conn,
                                created_gte,
                                limit,
                                offset,
                                service_id_mask,
                            );
                        }
                    }
                }
                Ok(res)
            })
    }

    pub fn list_where_created_gte_and_lte(
        conn: &PgConnection,
        created_gte: &DateTime<Utc>,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        Audit::list_where_created_gte_and_lte_inner(
            conn,
            created_gte,
            created_lte,
            limit,
            offset,
            service_id_mask,
        )
        .and_then(|res| {
            if let Some(offset_id) = offset_id {
                for (i, id) in res.iter().enumerate() {
                    if id == &offset_id {
                        let offset: i64 = (i + 1).try_into().unwrap();
                        return Audit::list_where_created_gte_and_lte_inner(
                            conn,
                            created_gte,
                            created_lte,
                            limit,
                            offset,
                            service_id_mask,
                        );
                    }
                }
            }
            Ok(res)
        })
    }

    // TODO(refactor): Remove use of AuditCreate here (replace with Insert).
    pub fn create(conn: &PgConnection, data: &AuditCreate) -> Result<Audit, DriverError> {
        use crate::driver::postgres::schema::auth_audit::dsl::*;

        let now = Utc::now();
        let value = AuditInsert {
            created_at: &now,
            audit_id: Uuid::new_v4(),
            audit_user_agent: data.meta.user_agent(),
            audit_remote: data.meta.remote(),
            audit_forwarded: data.meta.forwarded(),
            audit_path: data.path,
            audit_data: data.data,
            key_id: data.key_id,
            service_id: data.service_id,
            user_id: data.user_id,
            user_key_id: data.user_key_id,
        };
        diesel::insert_into(auth_audit)
            .values(&value)
            .get_result::<Audit>(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn read_by_id(
        conn: &PgConnection,
        id: Uuid,
        service_id_mask: Option<Uuid>,
    ) -> Result<Option<Audit>, DriverError> {
        use crate::driver::postgres::schema::auth_audit::dsl::*;

        match service_id_mask {
            Some(service_id_mask) => auth_audit
                .filter(service_id.eq(service_id_mask).and(audit_id.eq(id)))
                .get_result::<Audit>(conn),
            None => auth_audit.filter(audit_id.eq(id)).get_result::<Audit>(conn),
        }
        .map(Some)
        .or_else(|err| match err {
            diesel::result::Error::NotFound => Ok(None),
            _ => Err(DriverError::Diesel(err)),
        })
    }

    pub fn read_metrics(
        conn: &PgConnection,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<(String, i64)>, DriverError> {
        use crate::driver::postgres::schema::auth_audit::dsl::*;

        match service_id_mask {
            Some(service_id_mask) => auth_audit
                .select((audit_path, sql::<BigInt>("count(*)")))
                .group_by(audit_path)
                .filter(service_id.eq(service_id_mask))
                .order(audit_path.asc())
                .load(conn),
            None => auth_audit
                .select((audit_path, sql::<BigInt>("count(*)")))
                .group_by(audit_path)
                .order(audit_path.asc())
                .load(conn),
        }
        .map_err(DriverError::Diesel)
    }

    pub fn delete_by_created_at(
        conn: &PgConnection,
        audit_created_at: &DateTime<Utc>,
    ) -> Result<usize, DriverError> {
        use crate::driver::postgres::schema::auth_audit::dsl::*;

        diesel::delete(auth_audit.filter(created_at.le(audit_created_at)))
            .execute(conn)
            .map_err(DriverError::Diesel)
    }

    fn list_where_created_lte_inner(
        conn: &PgConnection,
        audit_created_lte: &DateTime<Utc>,
        limit: i64,
        offset: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        use crate::driver::postgres::schema::auth_audit::dsl::*;

        match service_id_mask {
            Some(service_id_mask) => auth_audit
                .select(audit_id)
                .filter(
                    service_id
                        .eq(service_id_mask)
                        .and(created_at.le(audit_created_lte)),
                )
                .limit(limit)
                .offset(offset)
                .order(created_at.desc())
                .load::<Uuid>(conn),
            None => auth_audit
                .select(audit_id)
                .filter(created_at.le(audit_created_lte))
                .limit(limit)
                .offset(offset)
                .order(created_at.desc())
                .load::<Uuid>(conn),
        }
        .map_err(DriverError::Diesel)
    }

    fn list_where_created_gte_inner(
        conn: &PgConnection,
        audit_created_gte: &DateTime<Utc>,
        limit: i64,
        offset: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        use crate::driver::postgres::schema::auth_audit::dsl::*;

        match service_id_mask {
            Some(service_id_mask) => auth_audit
                .select(audit_id)
                .filter(
                    service_id
                        .eq(service_id_mask)
                        .and(created_at.ge(audit_created_gte)),
                )
                .limit(limit)
                .offset(offset)
                .order(created_at.asc())
                .load::<Uuid>(conn),
            None => auth_audit
                .select(audit_id)
                .filter(created_at.ge(audit_created_gte))
                .limit(limit)
                .offset(offset)
                .order(created_at.asc())
                .load::<Uuid>(conn),
        }
        .map_err(DriverError::Diesel)
    }

    fn list_where_created_gte_and_lte_inner(
        conn: &PgConnection,
        audit_created_gte: &DateTime<Utc>,
        audit_created_lte: &DateTime<Utc>,
        limit: i64,
        offset: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        use crate::driver::postgres::schema::auth_audit::dsl::*;

        match service_id_mask {
            Some(service_id_mask) => auth_audit
                .select(audit_id)
                .filter(
                    service_id
                        .eq(service_id_mask)
                        .and(created_at.ge(audit_created_gte))
                        .and(created_at.le(audit_created_lte)),
                )
                .limit(limit)
                .offset(offset)
                .order(created_at.asc())
                .load::<Uuid>(conn),
            None => auth_audit
                .select(audit_id)
                .filter(
                    created_at
                        .ge(audit_created_gte)
                        .and(created_at.le(audit_created_lte)),
                )
                .limit(limit)
                .offset(offset)
                .order(created_at.asc())
                .load::<Uuid>(conn),
        }
        .map_err(DriverError::Diesel)
    }
}
