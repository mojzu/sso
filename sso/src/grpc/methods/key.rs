use crate::{
    grpc::{pb, util::*, validate, Server},
    *,
};
use tonic::Response;
use validator::{Validate, ValidationErrors};

impl Validate for pb::KeyListRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
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
    server: &Server,
    request: MethodRequest<KeyList>,
) -> MethodResponse<pb::KeyListReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyList);
        let res: Result<Vec<Key>, MethodError> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(MethodError::Unauthorised)?;

            driver
                .as_ref()
                .key_list(&req, service.map(|s| s.id))
                .map_err(MethodError::BadRequest)
        };
        let data = audit_result_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::KeyListReply {
            meta: Some(req.into()),
            data: data.into_iter().map::<pb::Key, _>(|x| x.into()).collect(),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::KeyCreateRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::key_type(e, "type", self.r#type);
            validate::name(e, "name", &self.name);
            validate::uuid_opt(e, "service_id", self.service_id.as_ref().map(|x| &**x));
            validate::uuid_opt(e, "user_id", self.user_id.as_ref().map(|x| &**x));
        })
    }
}

pub async fn create(
    server: &Server,
    request: MethodRequest<KeyCreate>,
) -> MethodResponse<pb::KeyCreateReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyCreate);
        let res: Result<KeyWithValue, MethodError> = {
            // If service ID is some, root key is required to create service keys.
            match req.service_id {
                Some(service_id) => {
                    pattern::key_root_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                        .map_err(MethodError::Unauthorised)
                        .and_then(|_| {
                            match req.user_id {
                                // User ID is defined, creating user key for service.
                                Some(user_id) => driver.key_create(&KeyCreate::user(
                                    req.is_enabled,
                                    req.type_,
                                    req.name,
                                    service_id,
                                    user_id,
                                )),
                                // Creating service key.
                                None => driver.key_create(&KeyCreate::service(
                                    req.is_enabled,
                                    req.name,
                                    service_id,
                                )),
                            }
                            .map_err(MethodError::BadRequest)
                        })
                }
                None => {
                    pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                        .map_err(MethodError::Unauthorised)
                        .and_then(|service| {
                            match req.user_id {
                                // User ID is defined, creating user key for service.
                                Some(user_id) => driver.key_create(&KeyCreate::user(
                                    req.is_enabled,
                                    req.type_,
                                    req.name,
                                    service.id,
                                    user_id,
                                )),
                                // Service cannot create service keys.
                                None => Err(DriverError::ServiceCannotCreateServiceKey),
                            }
                            .map_err(MethodError::BadRequest)
                        })
                }
            }
        };
        let data = audit_result_subject(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::KeyCreateReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::KeyReadRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "id", &self.id);
            validate::uuid_opt(e, "user_id", self.user_id.as_ref().map(|x| &**x));
        })
    }
}

pub async fn read(
    server: &Server,
    request: MethodRequest<KeyRead>,
) -> MethodResponse<pb::KeyReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyRead);
        let res: Result<Key, MethodError> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(MethodError::Unauthorised)?;

            read_inner(driver.as_ref().as_ref(), &req, service.as_ref())
        };
        let data = audit_result_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::KeyReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::KeyUpdateRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "id", &self.id);
            validate::name_opt(e, "name", self.name.as_ref().map(|x| &**x));
        })
    }
}

pub async fn update(
    server: &Server,
    request: MethodRequest<KeyUpdate>,
) -> MethodResponse<pb::KeyReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyUpdate);
        let res: Result<(Key, Key), MethodError> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(MethodError::Unauthorised)?;

            let read = KeyRead::IdUser(req.id, None);
            let previous_key = read_inner(driver.as_ref().as_ref(), &read, service.as_ref())?;
            let key = driver
                .key_update(&KeyUpdate {
                    id: req.id,
                    is_enabled: req.is_enabled,
                    is_revoked: None,
                    name: req.name,
                })
                .map_err(MethodError::BadRequest)?;
            Ok((previous_key, key))
        };
        let data = audit_result_diff(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::KeyReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn delete(server: &Server, request: MethodRequest<KeyRead>) -> MethodResponse<()> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyDelete);
        let res: Result<Key, MethodError> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(MethodError::Unauthorised)?;

            let key = read_inner(driver.as_ref().as_ref(), &req, service.as_ref())?;
            driver
                .key_delete(&key.id)
                .map_err(MethodError::BadRequest)
                .map(|_| key)
        };
        audit_result_subject(driver.as_ref().as_ref(), &audit, res)?;
        Ok(())
    })
    .await?;
    Ok(Response::new(reply))
}

fn read_inner(driver: &dyn Driver, read: &KeyRead, service: Option<&Service>) -> MethodResult<Key> {
    driver
        .key_read(&read, service.map(|x| x.id))
        .map_err(MethodError::BadRequest)?
        .ok_or_else(|| MethodError::NotFound(DriverError::KeyNotFound))
        .map(|x| x.into())
}
