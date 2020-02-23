use crate::{
    grpc::{pb, util::*, GrpcServer},
    *,
};
use chrono::Utc;

impl validator::Validate for pb::AuditListRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        Validate::wrap(|e| {
            Validate::limit_opt(e, "limit", self.limit);
            Validate::uuid_opt(e, "offset_id", self.offset_id.as_ref().map(|x| &**x));
            Validate::uuid_vec(e, "id", &self.id);
            Validate::audit_type_vec(e, "type", &self.r#type);
            Validate::audit_subject_vec(e, "subject", &self.subject);
            Validate::uuid_vec(e, "service_id", &self.service_id);
            Validate::uuid_vec(e, "user_id", &self.user_id);
        })
    }
}

impl From<pb::AuditListRequest> for AuditList {
    fn from(x: pb::AuditListRequest) -> Self {
        let limit = x.limit.unwrap_or(DEFAULT_LIMIT);
        let ge = pb::timestamp_opt_to_datetime_opt(x.ge);
        let le = pb::timestamp_opt_to_datetime_opt(x.le);
        let offset_id = pb::string_opt_to_uuid_opt(x.offset_id);
        let query = match (ge, le) {
            (Some(ge), Some(le)) => AuditListQuery::CreatedLeAndGe(le, ge, limit, offset_id),
            (Some(ge), None) => AuditListQuery::CreatedGe(ge, limit, offset_id),
            (None, Some(le)) => AuditListQuery::CreatedLe(le, limit, offset_id),
            (None, None) => AuditListQuery::CreatedLe(Utc::now(), limit, offset_id),
        };
        let filter = AuditListFilter {
            id: pb::string_vec_to_uuid_vec_opt(x.id),
            type_: pb::string_vec_to_string_vec_opt(x.r#type),
            subject: pb::string_vec_to_string_vec_opt(x.subject),
            service_id: pb::string_vec_to_uuid_vec_opt(x.service_id),
            user_id: pb::string_vec_to_uuid_vec_opt(x.user_id),
        };
        AuditList { query, filter }
    }
}

pub async fn list(
    server: &GrpcServer,
    request: GrpcMethodRequest<AuditList>,
) -> GrpcMethodResult<pb::AuditListReply> {
    let (audit_meta, auth, req) = request.into_inner();
    let driver = server.driver();

    blocking_method(move || {
        let data = audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuditList,
            |driver, audit| {
                let service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                driver
                    .audit_list(&req, service.map(|s| s.id))
                    .map_err(GrpcMethodError::BadRequest)
            },
        )?;
        Ok((data, req))
    })
    .await
    .map(|(data, req)| pb::AuditListReply {
        meta: Some(req.into()),
        data: data.into_iter().map::<pb::Audit, _>(|x| x.into()).collect(),
    })
}

impl validator::Validate for pb::AuditCreateRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        Validate::wrap(|e| {
            Validate::audit_type(e, "type", &self.r#type);
            Validate::audit_subject_opt(e, "subject", self.subject.as_ref().map(|x| &**x));
            Validate::uuid_opt(e, "user_id", self.user_id.as_ref().map(|x| &**x));
            Validate::uuid_opt(e, "user_key_id", self.user_key_id.as_ref().map(|x| &**x));
        })
    }
}

pub async fn create(
    server: &GrpcServer,
    request: GrpcMethodRequest<pb::AuditCreateRequest>,
) -> GrpcMethodResult<pb::AuditReadReply> {
    let (audit_meta, auth, req) = request.into_inner();
    let data = pb::struct_opt_to_value_opt(req.data);
    let req = AuditCreate::new(audit_meta.clone(), req.r#type)
        .subject(req.subject)
        .data(data)
        .user_id(pb::string_opt_to_uuid_opt(req.user_id))
        .user_key_id(pb::string_opt_to_uuid_opt(req.user_key_id));
    let driver = server.driver();

    blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuditCreate,
            |driver, audit| {
                let _service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                audit
                    .create2(driver, &req)
                    .map_err(GrpcMethodError::BadRequest)
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|data| pb::AuditReadReply {
        data: Some(data.into()),
    })
}

impl validator::Validate for pb::AuditReadRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        Validate::wrap(|e| {
            Validate::uuid(e, "id", &self.id);
            Validate::audit_subject_opt(e, "subject", self.subject.as_ref().map(|x| &**x));
        })
    }
}

pub async fn read(
    server: &GrpcServer,
    request: GrpcMethodRequest<AuditRead>,
) -> GrpcMethodResult<pb::AuditReadReply> {
    let (audit_meta, auth, req) = request.into_inner();
    let driver = server.driver();

    blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuditRead,
            |driver, audit| {
                let service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                driver
                    .audit_read(&req, service.map(|x| x.id))
                    .map_err(GrpcMethodError::BadRequest)?
                    .ok_or_else(|| GrpcMethodError::NotFound(DriverError::AuditNotFound))
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|data| pb::AuditReadReply {
        data: Some(data.into()),
    })
}

impl validator::Validate for pb::AuditUpdateRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        Validate::wrap(|e| {
            Validate::uuid(e, "id", &self.id);
            Validate::audit_subject_opt(e, "subject", self.subject.as_ref().map(|x| &**x));
        })
    }
}

pub async fn update(
    server: &GrpcServer,
    request: GrpcMethodRequest<AuditUpdate>,
) -> GrpcMethodResult<pb::AuditReadReply> {
    let (audit_meta, auth, req) = request.into_inner();
    let driver = server.driver();

    blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuditUpdate,
            |driver, audit| {
                let service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                driver
                    .audit_update(&req, service.map(|x| x.id))
                    .map_err(GrpcMethodError::BadRequest)
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|data| pb::AuditReadReply {
        data: Some(data.into()),
    })
}
