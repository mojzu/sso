use crate::{
    api::{self, ApiError, ApiResult, ValidateRequest},
    grpc::{pb, util::*},
    *,
};
use std::convert::TryInto;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub async fn list(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::ServiceListRequest>,
) -> Result<Response<pb::ServiceListReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: ServiceList = request.into_inner().try_into()?;
    ServiceList::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::ServiceList);
        let res: Result<Vec<Service>, Status> = {
            pattern::key_root_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            driver
                .service_list(&req)
                .map_err(ApiError::BadRequest)
                .map_err::<Status, _>(Into::into)
        };
        let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::ServiceListReply {
            meta: Some(req.into()),
            data: data
                .into_iter()
                .map::<pb::Service, _>(|x| x.into())
                .collect(),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn create(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::ServiceCreateRequest>,
) -> Result<Response<pb::ServiceReadReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: ServiceCreate = request.into_inner().try_into()?;
    ServiceCreate::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::ServiceCreate);
        let res: Result<Service, Status> = {
            pattern::key_root_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            driver
                .service_create(&req)
                .map_err(ApiError::BadRequest)
                .map_err::<Status, _>(Into::into)
        };
        let data = api::result_audit_subject(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::ServiceReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn read(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::ServiceReadRequest>,
) -> Result<Response<pb::ServiceReadReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: ServiceRead = request.into_inner().try_into()?;
    ServiceRead::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::ServiceRead);
        let res: Result<Service, Status> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            read_inner(driver.as_ref().as_ref(), &req, service.map(|x| x.id))
        };
        let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::ServiceReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn update(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::ServiceUpdateRequest>,
) -> Result<Response<pb::ServiceReadReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: ServiceUpdate = request.into_inner().try_into()?;
    ServiceUpdate::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::ServiceUpdate);
        let res: Result<(Service, Service), Status> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            let read = ServiceRead::new(req.id);
            let previous_service =
                read_inner(driver.as_ref().as_ref(), &read, service.map(|x| x.id))?;
            let service = driver.service_update(&req).map_err(ApiError::BadRequest)?;
            Ok((previous_service, service))
        };
        let data = api::result_audit_diff(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::ServiceReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn delete(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::ServiceReadRequest>,
) -> Result<Response<()>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: ServiceRead = request.into_inner().try_into()?;
    ServiceRead::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::ServiceDelete);
        let res: Result<Service, Status> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            let service = read_inner(driver.as_ref().as_ref(), &req, service.map(|x| x.id))?;
            driver
                .service_delete(&service.id)
                .map_err(ApiError::BadRequest)
                .map_err::<tonic::Status, _>(Into::into)
                .map(|_| service)
        };
        api::result_audit_subject(driver.as_ref().as_ref(), &audit, res)?;
        Ok(())
    })
    .await?;
    Ok(Response::new(reply))
}

fn read_inner(
    driver: &dyn Driver,
    read: &ServiceRead,
    service_id: Option<Uuid>,
) -> ApiResult<Service> {
    driver
        .service_read(read, service_id)
        .map_err(ApiError::BadRequest)
        .map_err::<tonic::Status, _>(Into::into)?
        .ok_or_else(|| DriverError::ServiceNotFound)
        .map_err(ApiError::NotFound)
        .map_err::<tonic::Status, _>(Into::into)
}
