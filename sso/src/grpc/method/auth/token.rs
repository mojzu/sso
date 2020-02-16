use crate::{
    grpc::{method::auth::api_csrf_verify, pb, util::*, validate, Server},
    *,
};
use validator::{Validate, ValidationErrors};

impl Validate for pb::AuthTokenRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::token(e, "token", &self.token);
            validate::audit_type_opt(e, "audit", self.audit.as_ref().map(|x| &**x));
        })
    }
}

pub async fn verify(
    server: &Server,
    request: MethodRequest<pb::AuthTokenRequest>,
) -> MethodResult<pb::AuthTokenVerifyReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    method_blocking(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthTokenVerify,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(MethodError::Unauthorised)?;

                // Unsafely decode token to get user identifier, used to read key for safe token decode.
                let (user_id, _) =
                    Jwt::decode_unsafe(&req.token, service.id).map_err(MethodError::BadRequest)?;

                // Token verify requires token key type.
                let user = pattern::user_read_id_checked(driver, Some(&service), audit, user_id)
                    .map_err(MethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
                        .map_err(MethodError::BadRequest)?;

                // Safely decode token with user key.
                let access_token_expires =
                    Jwt::decode_access_token(&service, &user, &key, &req.token)
                        .map_err(MethodError::BadRequest)?;

                // Token verified.
                let user_token = UserTokenAccess {
                    user: user.clone(),
                    access_token: req.token.clone(),
                    access_token_expires,
                };

                // Optionally create custom audit log.
                if let Some(x) = &req.audit {
                    let audit = audit
                        .create(driver, x, None, None)
                        .map_err(MethodError::BadRequest)?;
                    Ok((user, user_token, Some(audit)))
                } else {
                    Ok((user, user_token, None))
                }
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|(user, token, audit)| pb::AuthTokenVerifyReply {
        user: Some(user.into()),
        access: Some(token.into()),
        audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
    })
}

pub async fn refresh(
    server: &Server,
    request: MethodRequest<pb::AuthTokenRequest>,
) -> MethodResult<pb::AuthTokenReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let access_token_expires = server.options().access_token_expires();
    let refresh_token_expires = server.options().refresh_token_expires();
    method_blocking(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthTokenRefresh,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(MethodError::Unauthorised)?;

                // Unsafely decode token to get user identifier, used to read key for safe token decode.
                let (user_id, _) =
                    Jwt::decode_unsafe(&req.token, service.id).map_err(MethodError::BadRequest)?;

                // Token refresh requires token key type.
                let user = pattern::user_read_id_checked(driver, Some(&service), audit, user_id)
                    .map_err(MethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
                        .map_err(MethodError::BadRequest)?;

                // Safely decode token with user key.
                let csrf_key = Jwt::decode_refresh_token(&service, &user, &key, &req.token)
                    .map_err(MethodError::BadRequest)?;

                // Verify CSRF to prevent reuse.
                api_csrf_verify(driver, &service, &csrf_key)?;

                // Encode user token.
                let user_token = Jwt::encode_user_token(
                    driver,
                    &service,
                    user,
                    &key,
                    access_token_expires,
                    refresh_token_expires,
                )
                .map_err(MethodError::BadRequest)?;

                // Optionally create custom audit log.
                if let Some(x) = &req.audit {
                    let audit = audit
                        .create(driver, x, None, None)
                        .map_err(MethodError::BadRequest)?;
                    Ok((user_token, Some(audit)))
                } else {
                    Ok((user_token, None))
                }
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|(user_token, audit)| pb::AuthTokenReply {
        user: Some(user_token.user.clone().into()),
        access: Some(user_token.access_token()),
        refresh: Some(user_token.refresh_token()),
        audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
    })
}

pub async fn revoke(
    server: &Server,
    request: MethodRequest<pb::AuthTokenRequest>,
) -> MethodResult<pb::AuthAuditReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    method_blocking(move || {
        audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthTokenRevoke,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(MethodError::Unauthorised)?;

                // Unsafely decode token to get user identifier, used to read key for safe token decode.
                let (user_id, token_type) =
                    Jwt::decode_unsafe(&req.token, service.id).map_err(MethodError::BadRequest)?;

                // Token revoke requires token key type.
                // Do not check user, key is enabled or not revoked.
                let user = pattern::user_read_id_unchecked(driver, Some(&service), audit, user_id)
                    .map_err(MethodError::BadRequest)?;
                let key = pattern::key_read_user_unchecked(
                    driver,
                    &service,
                    audit,
                    &user,
                    KeyType::Token,
                )
                .map_err(MethodError::BadRequest)?;

                // Safely decode token with user key.
                let csrf_key = Jwt::decode_csrf_key(&service, &user, &key, token_type, &req.token)
                    .map_err(MethodError::BadRequest)?;
                if let Some(csrf_key) = csrf_key {
                    driver
                        .csrf_read(&csrf_key)
                        .map_err(MethodError::BadRequest)?;
                }

                // Token revoked, disable and revoke linked key.
                driver
                    .key_update(&KeyUpdate {
                        id: key.id,
                        is_enabled: Some(false),
                        is_revoked: Some(true),
                        name: None,
                    })
                    .map_err(MethodError::BadRequest)?;

                // Optionally create custom audit log.
                if let Some(x) = &req.audit {
                    let audit = audit
                        .create(driver, x, None, None)
                        .map_err(MethodError::BadRequest)?;
                    Ok(Some(audit))
                } else {
                    Ok(None)
                }
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|audit| pb::AuthAuditReply {
        audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
    })
}

pub async fn exchange(
    server: &Server,
    request: MethodRequest<pb::AuthTokenRequest>,
) -> MethodResult<pb::AuthTokenReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let access_token_expires = server.options().access_token_expires();
    let refresh_token_expires = server.options().refresh_token_expires();
    method_blocking(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthTokenExchange,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(MethodError::Unauthorised)?;

                // Unsafely decode token to get user identifier, used to read key for safe token decode.
                let (user_id, source_service_id) =
                    Jwt::decode_unsafe_service_id(&req.token, service.id)
                        .map_err(MethodError::BadRequest)?;

                // Read token source service.
                let source_service = driver
                    .service_read(&ServiceRead::new(source_service_id), None)
                    .map_err(MethodError::BadRequest)?
                    .ok_or_else(|| DriverError::ServiceNotFound)
                    .map_err(MethodError::BadRequest)?;

                // Token refresh requires token key type.
                // Read user key for source service to safely decode token.
                let user =
                    pattern::user_read_id_checked(driver, Some(&source_service), audit, user_id)
                        .map_err(MethodError::BadRequest)?;
                let target_key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
                        .map_err(MethodError::BadRequest)?;
                let source_key = pattern::key_read_user_checked(
                    driver,
                    &source_service,
                    audit,
                    &user,
                    KeyType::Token,
                )
                .map_err(MethodError::BadRequest)?;

                // Safely decode token with user key.
                let csrf_key =
                    Jwt::decode_refresh_token(&source_service, &user, &source_key, &req.token)
                        .map_err(MethodError::BadRequest)?;

                // Verify CSRF, and recreate so source service can still reuse refresh token.
                let csrf = api_csrf_verify(driver, &source_service, &csrf_key)?;
                driver
                    .csrf_create(&CsrfCreate::copy(csrf))
                    .map_err(MethodError::BadRequest)?;

                // Encode user token with target service key.
                let user_token = Jwt::encode_user_token(
                    driver,
                    &service,
                    user,
                    &target_key,
                    access_token_expires,
                    refresh_token_expires,
                )
                .map_err(MethodError::BadRequest)?;

                // Optionally create custom audit log.
                if let Some(x) = &req.audit {
                    let audit = audit
                        .create(driver, x, None, None)
                        .map_err(MethodError::BadRequest)?;
                    Ok((user_token, Some(audit)))
                } else {
                    Ok((user_token, None))
                }
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|(user_token, audit)| pb::AuthTokenReply {
        user: Some(user_token.user.clone().into()),
        access: Some(user_token.access_token()),
        refresh: Some(user_token.refresh_token()),
        audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
    })
}
