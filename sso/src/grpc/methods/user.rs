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
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: UserRead = request.into_inner().try_into()?;
    // TODO(refactor): Validate input.
    // UserRead::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::UserRead);
        let res: Result<User, Status> = {
            let _service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            read_inner(driver.as_ref().as_ref(), &req)
        };
        let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::UserReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn update(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::UserUpdateRequest>,
) -> Result<Response<pb::UserReadReply>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: UserUpdate = request.into_inner().try_into()?;
    UserUpdate::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::UserUpdate);
        let res: Result<(User, User), Status> = {
            let _service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            let read = UserRead::Id(req.id);
            let previous_user = read_inner(driver.as_ref().as_ref(), &read)?;

            let user = driver.user_update(&req).map_err(ApiError::BadRequest)?;
            Ok((previous_user, user))
        };
        let data = api::result_audit_diff(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::UserReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn delete(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::UserReadRequest>,
) -> Result<Response<()>, Status> {
    let (audit_meta, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
    let req: UserRead = request.into_inner().try_into()?;
    // TODO(refactor): Validate input.
    // UserRead::status_validate(&req)?;

    let driver = driver.clone();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::UserDelete);
        let res: Result<User, Status> = {
            let _service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(ApiError::Unauthorised)?;

            let user = read_inner(driver.as_ref().as_ref(), &req)?;
            driver
                .user_delete(&user.id)
                .map_err(ApiError::BadRequest)
                .map_err::<tonic::Status, _>(Into::into)
                .map(|_| user)
        };
        api::result_audit_subject(driver.as_ref().as_ref(), &audit, res)?;
        Ok(())
    })
    .await?;
    Ok(Response::new(reply))
}

fn read_inner(driver: &dyn Driver, read: &UserRead) -> ApiResult<User> {
    driver
        .user_read(read)
        .map_err(ApiError::BadRequest)
        .map_err::<tonic::Status, _>(Into::into)?
        .ok_or_else(|| DriverError::UserNotFound)
        .map_err(ApiError::NotFound)
        .map_err::<tonic::Status, _>(Into::into)
}
