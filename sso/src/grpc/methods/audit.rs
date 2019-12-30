use crate::{
    api::{self, ApiError, ValidateRequest},
    grpc::{pb, util::*},
    *,
};
use std::convert::TryInto;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub async fn list(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::AuditListRequest>,
) -> Result<Response<pb::AuditListReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: AuditList = request.into_inner().try_into()?;
    AuditList::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditList);
        let res: Result<Vec<Audit>, Status> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            driver
                .as_ref()
                .audit_list(&req, service.map(|s| s.id))
                .map_err(ApiError::BadRequest)
                .map_err::<Status, _>(Into::into)
        };
        let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuditListReply {
            meta: Some(req.into()),
            data: data.into_iter().map::<pb::Audit, _>(|x| x.into()).collect(),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn create(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::AuditCreateRequest>,
) -> Result<Response<pb::AuditReadReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req = request.into_inner();
    let data = struct_opt_to_value_opt(req.data);
    let req = AuditCreate::new(audit_meta.clone(), req.r#type)
        .subject(req.subject)
        .data(data)
        .user_id(string_opt_to_uuid_opt(req.user_id)?)
        .user_key_id(string_opt_to_uuid_opt(req.user_key_id)?);
    AuditCreate::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditCreate);
        let res: Result<Audit, Status> = {
            let _service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            audit
                .create2(driver.as_ref().as_ref(), req)
                .map_err(ApiError::BadRequest)
                .map_err::<Status, _>(Into::into)
        };
        let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuditReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn read(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::AuditReadRequest>,
) -> Result<Response<pb::AuditReadReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: AuditRead = request.into_inner().try_into()?;
    AuditRead::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditRead);
        let res: Result<Audit, Status> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            driver
                .audit_read(&req, service.map(|x| x.id))
                .map_err(ApiError::BadRequest)
                .map_err::<Status, _>(Into::into)?
                .ok_or_else(|| {
                    let e: Status = ApiError::NotFound(DriverError::AuditNotFound).into();
                    e
                })
        };
        let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuditReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn update(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::AuditUpdateRequest>,
) -> Result<Response<pb::AuditReadReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: AuditUpdate = request.into_inner().try_into()?;
    AuditUpdate::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditUpdate);
        let res: Result<Audit, Status> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            driver
                .audit_update(&req, service.map(|x| x.id))
                .map_err(ApiError::BadRequest)
                .map_err::<Status, _>(Into::into)
        };
        let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuditReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}
