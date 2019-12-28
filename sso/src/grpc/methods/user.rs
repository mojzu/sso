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
    request: Request<pb::UserListRequest>,
) -> Result<Response<pb::UserListReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: UserList = request.into_inner().try_into()?;
    UserList::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::UserList);
        let res: Result<Vec<User>, Status> = {
            let _service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            driver
                .user_list(&req)
                .map_err(ApiError::BadRequest)
                .map_err::<Status, _>(Into::into)
        };
        let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::UserListReply {
            meta: Some(req.into()),
            data: data.into_iter().map::<pb::User, _>(|x| x.into()).collect(),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn create(
    driver: Arc<Box<dyn Driver>>,
    client: Arc<reqwest::Client>,
    password_pwned_enabled: bool,
    request: Request<pb::UserCreateRequest>,
) -> Result<Response<pb::UserCreateReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req = request.into_inner();
    let password = req.password.clone();
    let req: UserCreate = req.try_into()?;
    UserCreate::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::UserCreate);
        let password_meta = api::password_meta(client.as_ref(), password_pwned_enabled, password)
            .map_err(ApiError::BadRequest)?;

        let res: Result<User, Status> = {
            let _service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            driver
                .user_create(&req)
                .map_err(ApiError::BadRequest)
                .map_err::<tonic::Status, _>(Into::into)
        };
        let data = api::result_audit_subject(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::UserCreateReply {
            meta: Some(password_meta.into()),
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn read(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::UserReadRequest>,
) -> Result<Response<pb::UserReadReply>, Status> {
    unimplemented!();
}

pub async fn update(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::UserUpdateRequest>,
) -> Result<Response<pb::UserReadReply>, Status> {
    unimplemented!();
}

pub async fn delete(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::UserReadRequest>,
) -> Result<Response<()>, Status> {
    unimplemented!();
}
