use crate::prelude::*;

impl validator::Validate for pb::ServiceListRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid_opt(e, "gt", self.gt.as_ref().map(|x| &**x));
            validate::uuid_opt(e, "lt", self.lt.as_ref().map(|x| &**x));
            validate::limit_opt(e, "limit", self.limit);
            validate::uuid_vec(e, "id", &self.id);
        })
    }
}

pub async fn list(
    server: &GrpcServer,
    request: GrpcMethodRequest<ServiceList>,
) -> GrpcMethodResult<pb::ServiceListReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        let data = audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::ServiceList,
            |driver, audit| {
                let service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                driver
                    .service_list(&req, service.map(|x| x.id))
                    .map_err(GrpcMethodError::BadRequest)
            },
        )?;
        Ok((req, data))
    })
    .await
    .map(|(req, data)| pb::ServiceListReply {
        meta: Some(req.into()),
        data: data
            .into_iter()
            .map::<pb::Service, _>(|x| x.into())
            .collect(),
    })
}

impl validator::Validate for pb::ServiceCreateRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        validate::wrap(|e| {
            validate::name(e, "name", &self.name);
            validate::url(e, "url", &self.url);
            validate::text_opt(
                e,
                "user_email_text",
                self.user_email_text.as_ref().map(|x| &**x),
            );
            validate::url_opt(
                e,
                "provider_local_url",
                self.provider_local_url.as_ref().map(|x| &**x),
            );
            validate::url_opt(
                e,
                "provider_github_oauth2_url",
                self.provider_github_oauth2_url.as_ref().map(|x| &**x),
            );
            validate::url_opt(
                e,
                "provider_microsoft_oauth2_url",
                self.provider_microsoft_oauth2_url.as_ref().map(|x| &**x),
            );
        })
    }
}

pub async fn create(
    server: &GrpcServer,
    request: GrpcMethodRequest<ServiceCreate>,
) -> GrpcMethodResult<pb::ServiceReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_subject(
            driver.as_ref(),
            audit_meta,
            AuditType::ServiceCreate,
            |driver, audit| {
                pattern::key_root_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                driver
                    .service_create(&req)
                    .map_err(GrpcMethodError::BadRequest)
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|data| pb::ServiceReadReply {
        data: Some(data.into()),
    })
}

impl validator::Validate for pb::ServiceReadRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "id", &self.id);
        })
    }
}

pub async fn read(
    server: &GrpcServer,
    request: GrpcMethodRequest<ServiceRead>,
) -> GrpcMethodResult<pb::ServiceReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::ServiceRead,
            |driver, audit| {
                let service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                read_inner(driver, &req, service.map(|x| x.id))
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|data| pb::ServiceReadReply {
        data: Some(data.into()),
    })
}

impl validator::Validate for pb::ServiceUpdateRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "id", &self.id);
            validate::name_opt(e, "name", self.name.as_ref().map(|x| &**x));
            validate::url_opt(e, "url", self.url.as_ref().map(|x| &**x));
            validate::text_opt(
                e,
                "user_email_text",
                self.user_email_text.as_ref().map(|x| &**x),
            );
            validate::url_opt(
                e,
                "provider_local_url",
                self.provider_local_url.as_ref().map(|x| &**x),
            );
            validate::url_opt(
                e,
                "provider_github_oauth2_url",
                self.provider_github_oauth2_url.as_ref().map(|x| &**x),
            );
            validate::url_opt(
                e,
                "provider_microsoft_oauth2_url",
                self.provider_microsoft_oauth2_url.as_ref().map(|x| &**x),
            );
        })
    }
}

pub async fn update(
    server: &GrpcServer,
    request: GrpcMethodRequest<ServiceUpdate>,
) -> GrpcMethodResult<pb::ServiceReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_diff(
            driver.as_ref(),
            audit_meta,
            AuditType::ServiceUpdate,
            |driver, audit| {
                let service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                let read = ServiceRead::new(req.id);
                let previous_service = read_inner(driver, &read, service.map(|x| x.id))?;
                let service = driver
                    .service_update(&req)
                    .map_err(GrpcMethodError::BadRequest)?;
                Ok((previous_service, service))
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|data| pb::ServiceReadReply {
        data: Some(data.into()),
    })
}

pub async fn delete(
    server: &GrpcServer,
    request: GrpcMethodRequest<ServiceRead>,
) -> GrpcMethodResult<()> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_subject(
            driver.as_ref(),
            audit_meta,
            AuditType::ServiceDelete,
            |driver, audit| {
                let service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                let service = read_inner(driver, &req, service.map(|x| x.id))?;
                driver
                    .service_delete(&service.id)
                    .map_err(GrpcMethodError::BadRequest)
                    .map(|_| service)
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|_data| ())
}

fn read_inner(
    driver: &Postgres,
    read: &ServiceRead,
    service_id: Option<Uuid>,
) -> GrpcMethodResult<Service> {
    driver
        .service_read(read, service_id)
        .map_err(GrpcMethodError::BadRequest)?
        .ok_or_else(|| DriverError::ServiceNotFound)
        .map_err(GrpcMethodError::NotFound)
}
