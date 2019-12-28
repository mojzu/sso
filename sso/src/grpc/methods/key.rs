use crate::{
    api::{self, ApiError, ApiResult, ValidateRequest},
    grpc::{pb, util::*},
    *,
};
use std::convert::TryInto;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub async fn list(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::KeyListRequest>,
) -> Result<Response<pb::KeyListReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: KeyList = request.into_inner().try_into()?;
    KeyList::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyList);
        let res: Result<Vec<Key>, Status> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            driver
                .as_ref()
                .key_list(&req, service.map(|s| s.id))
                .map_err(ApiError::BadRequest)
                .map_err::<Status, _>(Into::into)
        };
        let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::KeyListReply {
            meta: Some(req.into()),
            data: data.into_iter().map::<pb::Key, _>(|x| x.into()).collect(),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn create(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::KeyCreateRequest>,
) -> Result<Response<pb::KeyCreateReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: KeyCreate = request.into_inner().try_into()?;
    KeyCreate::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyCreate);
        let res: Result<KeyWithValue, Status> = {
            // If service ID is some, root key is required to create service keys.
            match req.service_id {
                Some(service_id) => {
                    pattern::key_root_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                        .map_err(ApiError::Unauthorised)
                        .map_err::<tonic::Status, _>(Into::into)
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
                            .map_err(ApiError::BadRequest)
                            .map_err::<tonic::Status, _>(Into::into)
                        })
                }
                None => {
                    pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                        .map_err(ApiError::Unauthorised)
                        .map_err::<tonic::Status, _>(Into::into)
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
                            .map_err(ApiError::BadRequest)
                            .map_err::<tonic::Status, _>(Into::into)
                        })
                }
            }
        };
        let data = api::result_audit_subject(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::KeyCreateReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn read(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::KeyReadRequest>,
) -> Result<Response<pb::KeyReadReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: KeyRead = request.into_inner().try_into()?;
    // TODO(refactor): Validate input.
    // KeyRead::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyRead);
        let res: Result<Key, Status> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)
                .map_err::<tonic::Status, _>(Into::into)?;

            read_inner(driver.as_ref().as_ref(), &req, service.as_ref())
        };
        let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::KeyReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn update(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::KeyUpdateRequest>,
) -> Result<Response<pb::KeyReadReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: KeyUpdate = request.into_inner().try_into()?;
    KeyUpdate::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyUpdate);
        let res: Result<(Key, Key), Status> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)
                .map_err::<tonic::Status, _>(Into::into)?;

            let read = KeyRead::IdUser(req.id, None);
            let previous_key = read_inner(driver.as_ref().as_ref(), &read, service.as_ref())?;
            let key = driver
                .key_update(&KeyUpdate {
                    id: req.id,
                    is_enabled: req.is_enabled,
                    is_revoked: None,
                    name: req.name,
                })
                .map_err(ApiError::BadRequest)?;
            Ok((previous_key, key))
        };
        let data = api::result_audit_diff(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::KeyReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn delete(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::KeyReadRequest>,
) -> Result<Response<()>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: KeyRead = request.into_inner().try_into()?;
    // TODO(refactor): Validate input.
    // KeyRead::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyDelete);
        let res: Result<Key, Status> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            let key = read_inner(driver.as_ref().as_ref(), &req, service.as_ref())?;
            driver
                .key_delete(&key.id)
                .map_err(ApiError::BadRequest)
                .map_err::<tonic::Status, _>(Into::into)
                .map(|_| key)
        };
        api::result_audit_subject(driver.as_ref().as_ref(), &audit, res)?;
        Ok(())
    })
    .await?;
    Ok(Response::new(reply))
}

fn read_inner(driver: &dyn Driver, read: &KeyRead, service: Option<&Service>) -> ApiResult<Key> {
    driver
        .key_read(&read, service.map(|x| x.id))
        .map_err(ApiError::BadRequest)
        .map_err::<tonic::Status, _>(Into::into)?
        .ok_or_else(|| {
            let e: tonic::Status = ApiError::NotFound(DriverError::KeyNotFound).into();
            e
        })
        .map(|x| x.into())
}
