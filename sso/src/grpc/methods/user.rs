use crate::grpc::{validate, Server};
use crate::{
    api::{self, ApiError, ApiResult},
    grpc::{pb, util::*},
    *,
};
use std::convert::TryInto;
use tonic::{Response, Status};
use validator::{Validate, ValidationErrors};

impl Validate for pb::UserListRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid_opt(e, "gt", self.gt.as_ref().map(|x| &**x));
            validate::uuid_opt(e, "lt", self.lt.as_ref().map(|x| &**x));
            validate::name_opt(e, "name_ge", self.name_ge.as_ref().map(|x| &**x));
            validate::name_opt(e, "name_le", self.name_le.as_ref().map(|x| &**x));
            validate::limit_opt(e, "limit", self.limit);
            validate::uuid_opt(e, "offset_id", self.offset_id.as_ref().map(|x| &**x));
            validate::uuid_vec(e, "id", &self.id);
            validate::email_vec(e, "email", &self.email);
        })
    }
}

pub async fn list(
    server: &Server,
    request: MetaRequest<pb::UserListRequest>,
) -> Result<Response<pb::UserListReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();
    let req: UserList = req.try_into()?;

    let driver = server.driver();
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

impl Validate for pb::UserCreateRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::name(e, "name", &self.name);
            validate::email(e, "email", &self.email);
            validate::locale_opt(e, "locale", self.locale.as_ref().map(|x| &**x));
            validate::timezone_opt(e, "timezone", self.timezone.as_ref().map(|x| &**x));
            validate::password_opt(e, "password", self.password.as_ref().map(|x| &**x));
        })
    }
}

pub async fn create(
    server: &Server,
    request: MetaRequest<pb::UserCreateRequest>,
) -> Result<Response<pb::UserCreateReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();
    let password = req.password.clone();
    let req: UserCreate = req.try_into()?;

    let driver = server.driver();
    let client = server.client();
    let password_pwned_enabled = server.options().password_pwned_enabled();
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

impl Validate for pb::UserReadRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "id", &self.id);
        })
    }
}

pub async fn read(
    server: &Server,
    request: MetaRequest<pb::UserReadRequest>,
) -> Result<Response<pb::UserReadReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();
    let req: UserRead = req.try_into()?;

    let driver = server.driver();
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

impl Validate for pb::UserUpdateRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "id", &self.id);
            validate::name_opt(e, "name", self.name.as_ref().map(|x| &**x));
            validate::locale_opt(e, "locale", self.locale.as_ref().map(|x| &**x));
            validate::timezone_opt(e, "timezone", self.timezone.as_ref().map(|x| &**x));
        })
    }
}

pub async fn update(
    server: &Server,
    request: MetaRequest<pb::UserUpdateRequest>,
) -> Result<Response<pb::UserReadReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();
    let req: UserUpdate = req.try_into()?;

    let driver = server.driver();
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
    server: &Server,
    request: MetaRequest<pb::UserReadRequest>,
) -> Result<Response<()>, Status> {
    let (audit_meta, auth, req) = request.into_inner();
    let req: UserRead = req.try_into()?;

    let driver = server.driver();
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
