use crate::{
    grpc::{pb, util::*, validate, Server},
    *,
};
use chrono::Utc;
use tonic::Response;
use validator::{Validate, ValidationErrors};

impl Validate for pb::AuditListRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::limit_opt(e, "limit", self.limit);
            validate::uuid_opt(e, "offset_id", self.offset_id.as_ref().map(|x| &**x));
            validate::uuid_vec(e, "id", &self.id);
            validate::audit_type_vec(e, "type", &self.r#type);
            validate::audit_subject_vec(e, "subject", &self.subject);
            validate::uuid_vec(e, "service_id", &self.service_id);
            validate::uuid_vec(e, "user_id", &self.user_id);
        })
    }
}

impl From<pb::AuditListRequest> for AuditList {
    fn from(x: pb::AuditListRequest) -> Self {
        let limit = x.limit.unwrap_or(DEFAULT_LIMIT);
        let ge = timestamp_opt_to_datetime_opt(x.ge);
        let le = timestamp_opt_to_datetime_opt(x.le);
        let offset_id = string_opt_to_uuid_opt(x.offset_id);
        let query = match (ge, le) {
            (Some(ge), Some(le)) => AuditListQuery::CreatedLeAndGe(le, ge, limit, offset_id),
            (Some(ge), None) => AuditListQuery::CreatedGe(ge, limit, offset_id),
            (None, Some(le)) => AuditListQuery::CreatedLe(le, limit, offset_id),
            (None, None) => AuditListQuery::CreatedLe(Utc::now(), limit, offset_id),
        };
        let filter = AuditListFilter {
            id: string_vec_to_uuid_vec_opt(x.id),
            type_: string_vec_to_string_vec_opt(x.r#type),
            subject: string_vec_to_string_vec_opt(x.subject),
            service_id: string_vec_to_uuid_vec_opt(x.service_id),
            user_id: string_vec_to_uuid_vec_opt(x.user_id),
        };
        AuditList { query, filter }
    }
}

pub async fn list(
    server: &Server,
    request: MethodRequest<AuditList>,
) -> MethodResponse<pb::AuditListReply> {
    let (audit_meta, auth, req) = request.into_inner();
    let driver = server.driver();

    let reply = blocking::<_, MethodError, _>(move || {
        let data = audit_result_err(
            driver.as_ref().as_ref(),
            audit_meta,
            AuditType::AuditList,
            |driver, audit| {
                let service = pattern::key_authenticate2(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                driver
                    .audit_list(&req, service.map(|s| s.id))
                    .map_err(MethodError::BadRequest)
            },
        )?;
        let reply = pb::AuditListReply {
            meta: Some(req.into()),
            data: data.into_iter().map::<pb::Audit, _>(|x| x.into()).collect(),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::AuditCreateRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::audit_type(e, "type", &self.r#type);
            validate::audit_subject_opt(e, "subject", self.subject.as_ref().map(|x| &**x));
            validate::uuid_opt(e, "user_id", self.user_id.as_ref().map(|x| &**x));
            validate::uuid_opt(e, "user_key_id", self.user_key_id.as_ref().map(|x| &**x));
        })
    }
}

pub async fn create(
    server: &Server,
    request: MethodRequest<pb::AuditCreateRequest>,
) -> MethodResponse<pb::AuditReadReply> {
    let (audit_meta, auth, req) = request.into_inner();
    let data = struct_opt_to_value_opt(req.data);
    let req = AuditCreate::new(audit_meta.clone(), req.r#type)
        .subject(req.subject)
        .data(data)
        .user_id(string_opt_to_uuid_opt(req.user_id))
        .user_key_id(string_opt_to_uuid_opt(req.user_key_id));

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let data = audit_result_err(
            driver.as_ref().as_ref(),
            audit_meta,
            AuditType::AuditCreate,
            |driver, audit| {
                let _service = pattern::key_authenticate2(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                audit.create2(driver, &req).map_err(MethodError::BadRequest)
            },
        )?;
        let reply = pb::AuditReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::AuditReadRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "id", &self.id);
            validate::audit_subject_opt(e, "subject", self.subject.as_ref().map(|x| &**x));
        })
    }
}

pub async fn read(
    server: &Server,
    request: MethodRequest<AuditRead>,
) -> MethodResponse<pb::AuditReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let data = audit_result_err(
            driver.as_ref().as_ref(),
            audit_meta,
            AuditType::AuditRead,
            |driver, audit| {
                let service = pattern::key_authenticate2(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                driver
                    .audit_read(&req, service.map(|x| x.id))
                    .map_err(MethodError::BadRequest)?
                    .ok_or_else(|| MethodError::NotFound(DriverError::AuditNotFound))
            },
        )?;
        let reply = pb::AuditReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::AuditUpdateRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "id", &self.id);
            validate::audit_subject_opt(e, "subject", self.subject.as_ref().map(|x| &**x));
        })
    }
}

pub async fn update(
    server: &Server,
    request: MethodRequest<AuditUpdate>,
) -> MethodResponse<pb::AuditReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let data = audit_result_err(
            driver.as_ref().as_ref(),
            audit_meta,
            AuditType::AuditUpdate,
            |driver, audit| {
                let service = pattern::key_authenticate2(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                driver
                    .audit_update(&req, service.map(|x| x.id))
                    .map_err(MethodError::BadRequest)
            },
        )?;
        let reply = pb::AuditReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}
