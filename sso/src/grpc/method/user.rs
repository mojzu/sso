use crate::{
    grpc::{pb, util::*, GrpcServer},
    *,
};

impl validator::Validate for pb::UserListRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        Validate::wrap(|e| {
            Validate::uuid_opt(e, "gt", self.gt.as_ref().map(|x| &**x));
            Validate::uuid_opt(e, "lt", self.lt.as_ref().map(|x| &**x));
            Validate::name_opt(e, "name_ge", self.name_ge.as_ref().map(|x| &**x));
            Validate::name_opt(e, "name_le", self.name_le.as_ref().map(|x| &**x));
            Validate::limit_opt(e, "limit", self.limit);
            Validate::uuid_opt(e, "offset_id", self.offset_id.as_ref().map(|x| &**x));
            Validate::uuid_vec(e, "id", &self.id);
            Validate::email_vec(e, "email", &self.email);
        })
    }
}

pub async fn list(
    server: &GrpcServer,
    request: GrpcMethodRequest<UserList>,
) -> GrpcMethodResult<pb::UserListReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        let data = audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::UserList,
            |driver, audit| {
                let _service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                driver.user_list(&req).map_err(GrpcMethodError::BadRequest)
            },
        )?;
        Ok((req, data))
    })
    .await
    .map(|(req, data)| pb::UserListReply {
        meta: Some(req.into()),
        data: data.into_iter().map::<pb::User, _>(|x| x.into()).collect(),
    })
}

impl validator::Validate for pb::UserCreateRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        Validate::wrap(|e| {
            Validate::name(e, "name", &self.name);
            Validate::email(e, "email", &self.email);
            Validate::locale_opt(e, "locale", self.locale.as_ref().map(|x| &**x));
            Validate::timezone_opt(e, "timezone", self.timezone.as_ref().map(|x| &**x));
            Validate::password_opt(e, "password", self.password.as_ref().map(|x| &**x));
        })
    }
}

pub async fn create(
    server: &GrpcServer,
    request: GrpcMethodRequest<pb::UserCreateRequest>,
) -> GrpcMethodResult<pb::UserCreateReply> {
    let (audit_meta, auth, req) = request.into_inner();
    let password = req.password.clone();
    let req: UserCreate = req.into();

    let client = server.client();
    let pwned_passwords = server.options().pwned_passwords_enabled();
    let password_meta = pattern::password_meta(client.as_ref(), pwned_passwords, password)
        .await
        .map_err(GrpcMethodError::BadRequest)?;

    let driver = server.driver();
    blocking_method(move || {
        let data = audit_result_subject(
            driver.as_ref(),
            audit_meta,
            AuditType::UserCreate,
            |driver, audit| {
                let _service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                driver
                    .user_create(&req)
                    .map_err(GrpcMethodError::BadRequest)
            },
        )?;
        Ok((password_meta, data))
    })
    .await
    .map(|(password_meta, data)| pb::UserCreateReply {
        meta: Some(password_meta.into()),
        data: Some(data.into()),
    })
}

impl validator::Validate for pb::UserReadRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        Validate::wrap(|e| {
            Validate::uuid(e, "id", &self.id);
        })
    }
}

pub async fn read(
    server: &GrpcServer,
    request: GrpcMethodRequest<UserRead>,
) -> GrpcMethodResult<pb::UserReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::UserRead,
            |driver, audit| {
                let _service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                read_inner(driver, &req)
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|data| pb::UserReadReply {
        data: Some(data.into()),
    })
}

impl validator::Validate for pb::UserUpdateRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        Validate::wrap(|e| {
            Validate::uuid(e, "id", &self.id);
            Validate::name_opt(e, "name", self.name.as_ref().map(|x| &**x));
            Validate::locale_opt(e, "locale", self.locale.as_ref().map(|x| &**x));
            Validate::timezone_opt(e, "timezone", self.timezone.as_ref().map(|x| &**x));
        })
    }
}

pub async fn update(
    server: &GrpcServer,
    request: GrpcMethodRequest<UserUpdate>,
) -> GrpcMethodResult<pb::UserReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_diff(
            driver.as_ref(),
            audit_meta,
            AuditType::UserUpdate,
            |driver, audit| {
                let _service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                let read = UserRead::Id(req.id);
                let previous_user = read_inner(driver, &read)?;

                let user = driver
                    .user_update(&req)
                    .map_err(GrpcMethodError::BadRequest)?;
                Ok((previous_user, user))
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|data| pb::UserReadReply {
        data: Some(data.into()),
    })
}

pub async fn delete(
    server: &GrpcServer,
    request: GrpcMethodRequest<UserRead>,
) -> GrpcMethodResult<()> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_subject(
            driver.as_ref(),
            audit_meta,
            AuditType::UserDelete,
            |driver, audit| {
                let _service = pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                let user = read_inner(driver, &req)?;
                driver
                    .user_delete(&user.id)
                    .map_err(GrpcMethodError::BadRequest)
                    .map(|_| user)
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|_data| ())
}

fn read_inner(driver: &Postgres, read: &UserRead) -> GrpcMethodResult<User> {
    driver
        .user_read(read)
        .map_err(GrpcMethodError::BadRequest)?
        .ok_or_else(|| DriverError::UserNotFound)
        .map_err(GrpcMethodError::NotFound)
}
