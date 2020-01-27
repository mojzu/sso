use crate::{
    grpc::{pb, util::*, validate, Server},
    *,
};
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
    request: MethodRequest<UserList>,
) -> MethodResult<pb::UserListReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking::<_, MethodError, _>(move || {
        let data = audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::UserList,
            |driver, audit| {
                let _service = pattern::key_authenticate(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                driver.user_list(&req).map_err(MethodError::BadRequest)
            },
        )?;
        let reply = pb::UserListReply {
            meta: Some(req.into()),
            data: data.into_iter().map::<pb::User, _>(|x| x.into()).collect(),
        };
        Ok(reply)
    })
    .await
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
    request: MethodRequest<pb::UserCreateRequest>,
) -> MethodResult<pb::UserCreateReply> {
    let (audit_meta, auth, req) = request.into_inner();
    let password = req.password.clone();
    let req: UserCreate = req.into();

    let driver = server.driver();
    let client = server.client();
    let password_pwned_enabled = server.options().password_pwned_enabled();
    blocking::<_, MethodError, _>(move || {
        let password_meta =
            pattern::password_meta(client.as_ref(), password_pwned_enabled, password)
                .map_err(MethodError::BadRequest)?;

        let data = audit_result_subject(
            driver.as_ref(),
            audit_meta,
            AuditType::UserCreate,
            |driver, audit| {
                let _service = pattern::key_authenticate(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                driver.user_create(&req).map_err(MethodError::BadRequest)
            },
        )?;
        let reply = pb::UserCreateReply {
            meta: Some(password_meta.into()),
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await
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
    request: MethodRequest<UserRead>,
) -> MethodResult<pb::UserReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking::<_, MethodError, _>(move || {
        let data = audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::UserRead,
            |driver, audit| {
                let _service = pattern::key_authenticate(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                read_inner(driver, &req)
            },
        )?;
        let reply = pb::UserReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await
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
    request: MethodRequest<UserUpdate>,
) -> MethodResult<pb::UserReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking::<_, MethodError, _>(move || {
        let data = audit_result_diff(
            driver.as_ref(),
            audit_meta,
            AuditType::UserUpdate,
            |driver, audit| {
                let _service = pattern::key_authenticate(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                let read = UserRead::Id(req.id);
                let previous_user = read_inner(driver, &read)?;

                let user = driver.user_update(&req).map_err(MethodError::BadRequest)?;
                Ok((previous_user, user))
            },
        )?;
        let reply = pb::UserReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await
}

pub async fn delete(server: &Server, request: MethodRequest<UserRead>) -> MethodResult<()> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking::<_, MethodError, _>(move || {
        audit_result_subject(
            driver.as_ref(),
            audit_meta,
            AuditType::UserDelete,
            |driver, audit| {
                let _service = pattern::key_authenticate(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                let user = read_inner(driver, &req)?;
                driver
                    .user_delete(&user.id)
                    .map_err(MethodError::BadRequest)
                    .map(|_| user)
            },
        )?;
        Ok(())
    })
    .await
}

fn read_inner(driver: &Postgres, read: &UserRead) -> MethodResult<User> {
    driver
        .user_read(read)
        .map_err(MethodError::BadRequest)?
        .ok_or_else(|| DriverError::UserNotFound)
        .map_err(MethodError::NotFound)
}
