use crate::prelude::*;

impl validator::Validate for pb::KeyListRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid_opt(e, "gt", self.gt.as_ref().map(|x| &**x));
            validate::uuid_opt(e, "lt", self.lt.as_ref().map(|x| &**x));
            validate::limit_opt(e, "limit", self.limit);
            validate::uuid_vec(e, "id", &self.id);
            validate::key_type_vec(e, "type", &self.r#type);
            validate::uuid_vec(e, "service_id", &self.service_id);
            validate::uuid_vec(e, "user_id", &self.user_id);
        })
    }
}

pub async fn list(
    server: &GrpcServer,
    request: GrpcMethodRequest<KeyList>,
) -> GrpcMethodResult<pb::KeyListReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        let data = audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::KeyList,
            |driver, audit| {
                let service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                driver
                    .key_list(&req, service.map(|s| s.id))
                    .map_err(GrpcMethodError::BadRequest)
            },
        )?;
        Ok((req, data))
    })
    .await
    .map(|(req, data)| pb::KeyListReply {
        meta: Some(req.into()),
        data: data.into_iter().map::<pb::Key, _>(|x| x.into()).collect(),
    })
}

impl validator::Validate for pb::KeyCreateRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        validate::wrap(|e| {
            validate::key_type(e, "type", self.r#type);
            validate::name(e, "name", &self.name);
            validate::uuid_opt(e, "service_id", self.service_id.as_ref().map(|x| &**x));
            validate::uuid_opt(e, "user_id", self.user_id.as_ref().map(|x| &**x));
        })
    }
}

pub async fn create(
    server: &GrpcServer,
    request: GrpcMethodRequest<KeyCreate>,
) -> GrpcMethodResult<pb::KeyCreateReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_subject(
            driver.as_ref(),
            audit_meta,
            AuditType::KeyCreate,
            |driver, audit| {
                // If service ID is some, root key is required to create service keys.
                match req.service_id {
                    Some(service_id) => {
                        pattern::key_root_authenticate(driver, audit, &auth)
                            .map_err(GrpcMethodError::Unauthorised)
                            .and_then(|_| {
                                match req.user_id {
                                    // User ID is defined, creating user key for service.
                                    Some(user_id) => driver.key_create(&KeyCreate::user(
                                        req.is_enabled,
                                        req.type_,
                                        &req.name,
                                        service_id,
                                        user_id,
                                    )),
                                    // Creating service key.
                                    None => driver.key_create(&KeyCreate::service(
                                        req.is_enabled,
                                        &req.name,
                                        service_id,
                                    )),
                                }
                                .map_err(GrpcMethodError::BadRequest)
                            })
                    }
                    None => {
                        pattern::key_service_authenticate(driver, audit, &auth)
                            .map_err(GrpcMethodError::Unauthorised)
                            .and_then(|service| {
                                match req.user_id {
                                    // User ID is defined, creating user key for service.
                                    Some(user_id) => driver.key_create(&KeyCreate::user(
                                        req.is_enabled,
                                        req.type_,
                                        &req.name,
                                        service.id,
                                        user_id,
                                    )),
                                    // Service cannot create service keys.
                                    None => Err(DriverError::ServiceCannotCreateServiceKey),
                                }
                                .map_err(GrpcMethodError::BadRequest)
                            })
                    }
                }
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|data| pb::KeyCreateReply {
        data: Some(data.into()),
    })
}

impl validator::Validate for pb::KeyReadRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "id", &self.id);
            validate::uuid_opt(e, "user_id", self.user_id.as_ref().map(|x| &**x));
        })
    }
}

pub async fn read(
    server: &GrpcServer,
    request: GrpcMethodRequest<KeyRead>,
) -> GrpcMethodResult<pb::KeyReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::KeyRead,
            |driver, audit| {
                let service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                read_inner(driver, &req, service.as_ref())
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|data| pb::KeyReadReply {
        data: Some(data.into()),
    })
}

impl validator::Validate for pb::KeyUpdateRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "id", &self.id);
            validate::name_opt(e, "name", self.name.as_ref().map(|x| &**x));
        })
    }
}

pub async fn update(
    server: &GrpcServer,
    request: GrpcMethodRequest<KeyUpdate>,
) -> GrpcMethodResult<pb::KeyReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_diff(
            driver.as_ref(),
            audit_meta,
            AuditType::KeyUpdate,
            |driver, audit| {
                let service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                let read = KeyRead::IdUser(req.id, None);
                let previous_key = read_inner(driver, &read, service.as_ref())?;
                let key = driver
                    .key_update(&KeyUpdate {
                        id: req.id,
                        is_enabled: req.is_enabled,
                        is_revoked: None,
                        name: req.name.clone(),
                    })
                    .map_err(GrpcMethodError::BadRequest)?;
                Ok((previous_key, key))
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|data| pb::KeyReadReply {
        data: Some(data.into()),
    })
}

pub async fn delete(
    server: &GrpcServer,
    request: GrpcMethodRequest<KeyRead>,
) -> GrpcMethodResult<()> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_subject(
            driver.as_ref(),
            audit_meta,
            AuditType::KeyDelete,
            |driver, audit| {
                let service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                let key = read_inner(driver, &req, service.as_ref())?;
                driver
                    .key_delete(&key.id)
                    .map_err(GrpcMethodError::BadRequest)
                    .map(|_| key)
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|_data| ())
}

fn read_inner(
    driver: &Postgres,
    read: &KeyRead,
    service: Option<&Service>,
) -> GrpcMethodResult<Key> {
    driver
        .key_read(&read, service.map(|x| x.id))
        .map_err(GrpcMethodError::BadRequest)?
        .ok_or_else(|| GrpcMethodError::NotFound(DriverError::KeyNotFound))
        .map(|x| x.into())
}
