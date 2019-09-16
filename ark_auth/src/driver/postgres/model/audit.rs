use crate::{driver::postgres::schema::auth_audit, Audit, AuditCreate, AuditList, DriverResult};
use chrono::{DateTime, Utc};
use diesel::{dsl::sql, prelude::*, sql_types::BigInt};
use serde_json::Value;
use std::convert::TryInto;
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_audit"]
#[primary_key(audit_id)]
pub struct ModelAudit {
    created_at: DateTime<Utc>,
    audit_id: Uuid,
    audit_user_agent: String,
    audit_remote: String,
    audit_forwarded: Option<String>,
    audit_path: String,
    audit_data: Value,
    key_id: Option<Uuid>,
    service_id: Option<Uuid>,
    user_id: Option<Uuid>,
    user_key_id: Option<Uuid>,
}

impl From<ModelAudit> for Audit {
    fn from(audit: ModelAudit) -> Self {
        Self {
            created_at: audit.created_at,
            id: audit.audit_id,
            user_agent: audit.audit_user_agent,
            remote: audit.audit_remote,
            forwarded: audit.audit_forwarded,
            path: audit.audit_path,
            data: audit.audit_data,
            key_id: audit.key_id,
            service_id: audit.service_id,
            user_id: audit.user_id,
            user_key_id: audit.user_key_id,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "auth_audit"]
struct ModelAuditInsert<'a> {
    created_at: &'a DateTime<Utc>,
    audit_id: &'a Uuid,
    audit_user_agent: &'a str,
    audit_remote: &'a str,
    audit_forwarded: Option<&'a str>,
    audit_path: &'a str,
    audit_data: &'a Value,
    key_id: Option<&'a Uuid>,
    service_id: Option<&'a Uuid>,
    user_id: Option<&'a Uuid>,
    user_key_id: Option<&'a Uuid>,
}

impl<'a> ModelAuditInsert<'a> {
    fn from_create(now: &'a DateTime<Utc>, id: &'a Uuid, create: &'a AuditCreate) -> Self {
        Self {
            created_at: now,
            audit_id: id,
            audit_user_agent: create.meta.user_agent(),
            audit_remote: create.meta.remote(),
            audit_forwarded: create.meta.forwarded(),
            audit_path: create.path,
            audit_data: create.data,
            key_id: create.key_id,
            service_id: create.service_id,
            user_id: create.user_id,
            user_key_id: create.user_key_id,
        }
    }
}

impl ModelAudit {
    pub fn list(conn: &PgConnection, list: &AuditList) -> DriverResult<Vec<Uuid>> {
        match list {
            AuditList::CreatedLe(l) => Self::list_where_created_le(conn),
            AuditList::CreatedGe(l) => Self::list_where_created_ge(conn),
            AuditList::CreatedLeAndGe(l) => Self::list_where_created_le_and_ge(conn),
        }
    }

    pub fn create(conn: &PgConnection, create: &AuditCreate) -> DriverResult<Audit> {
        use crate::driver::postgres::schema::auth_audit::dsl::*;

        let now = Utc::now();
        let id = Uuid::new_v4();
        let value = ModelAuditInsert::from_create(&now, &id, create);
        diesel::insert_into(auth_audit)
            .values(&value)
            .get_result::<ModelAudit>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn read_by_id(
        conn: &PgConnection,
        id: Uuid,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Option<Audit>> {
        use crate::driver::postgres::schema::auth_audit::dsl::*;

        match service_id_mask {
            Some(service_id_mask) => auth_audit
                .filter(service_id.eq(service_id_mask).and(audit_id.eq(id)))
                .get_result::<ModelAudit>(conn),
            None => auth_audit
                .filter(audit_id.eq(id))
                .get_result::<ModelAudit>(conn),
        }
        .optional()
        .map_err(Into::into)
        .map(|x| x.map(Into::into))
    }

    pub fn read_metrics(
        conn: &PgConnection,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<(String, i64)>> {
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
        .map_err(Into::into)
    }

    pub fn delete_by_created_at(
        conn: &PgConnection,
        audit_created_at: &DateTime<Utc>,
    ) -> DriverResult<usize> {
        use crate::driver::postgres::schema::auth_audit::dsl::*;

        diesel::delete(auth_audit.filter(created_at.le(audit_created_at)))
            .execute(conn)
            .map_err(Into::into)
    }

    fn list_where_created_le(
        conn: &PgConnection,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        ModelAudit::list_where_created_le_inner(conn, created_lte, limit, offset, service_id_mask)
            .and_then(|res| {
                if let Some(offset_id) = offset_id {
                    for (i, id) in res.iter().enumerate() {
                        if id == &offset_id {
                            let offset: i64 = (i + 1).try_into().unwrap();
                            return ModelAudit::list_where_created_le_inner(
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

    fn list_where_created_le_inner(
        conn: &PgConnection,
        audit_created_lte: &DateTime<Utc>,
        limit: i64,
        offset: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
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
        .map_err(Into::into)
    }

    fn list_where_created_ge(
        conn: &PgConnection,
        created_gte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        ModelAudit::list_where_created_ge_inner(conn, created_gte, limit, offset, service_id_mask)
            .and_then(|res| {
                if let Some(offset_id) = offset_id {
                    for (i, id) in res.iter().enumerate() {
                        if id == &offset_id {
                            let offset: i64 = (i + 1).try_into().unwrap();
                            return ModelAudit::list_where_created_ge_inner(
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

    fn list_where_created_ge_inner(
        conn: &PgConnection,
        audit_created_gte: &DateTime<Utc>,
        limit: i64,
        offset: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
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
        .map_err(Into::into)
    }

    fn list_where_created_le_and_ge(
        conn: &PgConnection,
        created_gte: &DateTime<Utc>,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        ModelAudit::list_where_created_le_and_ge_inner(
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
                        return ModelAudit::list_where_created_le_and_ge_inner(
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

    fn list_where_created_le_and_ge_inner(
        conn: &PgConnection,
        audit_created_gte: &DateTime<Utc>,
        audit_created_lte: &DateTime<Utc>,
        limit: i64,
        offset: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
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
        .map_err(Into::into)
    }
}
