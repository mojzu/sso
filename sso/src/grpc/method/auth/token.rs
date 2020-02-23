use crate::{
    grpc::{pb, util::*, GrpcServer},
    Jwt, *,
};

impl validator::Validate for pb::AuthTokenRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        Validate::wrap(|e| {
            Validate::token(e, "token", &self.token);
            Validate::audit_type_opt(e, "audit", self.audit.as_ref().map(|x| &**x));
        })
    }
}

pub async fn verify(
    server: &GrpcServer,
    request: GrpcMethodRequest<pb::AuthTokenRequest>,
) -> GrpcMethodResult<pb::AuthTokenVerifyReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthTokenVerify,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                // Unsafely decode token to get user identifier, used to read key for safe token decode.
                let (user_id, _) = Jwt::decode_unsafe_user(&req.token, service.id)
                    .map_err(GrpcMethodError::BadRequest)?;

                // Token verify requires token key type.
                let user = pattern::user_read_id_checked(driver, Some(&service), audit, user_id)
                    .map_err(GrpcMethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
                        .map_err(GrpcMethodError::BadRequest)?;

                // Safely decode token with user key.
                let access_token_expires = Jwt::decode_access(&service, &user, &key, &req.token)
                    .map_err(GrpcMethodError::BadRequest)?;

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
                        .map_err(GrpcMethodError::BadRequest)?;
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
        audit: pb::uuid_opt_to_string_opt(audit.map(|x| x.id)),
    })
}

pub async fn refresh(
    server: &GrpcServer,
    request: GrpcMethodRequest<pb::AuthTokenRequest>,
) -> GrpcMethodResult<pb::AuthTokenReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let access_token_expires = server.options().access_token_expires();
    let refresh_token_expires = server.options().refresh_token_expires();
    blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthTokenRefresh,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                // Unsafely decode token to get user identifier, used to read key for safe token decode.
                let (user_id, _) = Jwt::decode_unsafe_user(&req.token, service.id)
                    .map_err(GrpcMethodError::BadRequest)?;

                // Token refresh requires token key type.
                let user = pattern::user_read_id_checked(driver, Some(&service), audit, user_id)
                    .map_err(GrpcMethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
                        .map_err(GrpcMethodError::BadRequest)?;

                // Safely decode token with user key.
                let conn = driver.conn().map_err(GrpcMethodError::BadRequest)?;
                Jwt::decode_refresh(&conn, &service, &user, &key, &req.token)
                    .map_err(GrpcMethodError::BadRequest)?;

                // Encode user token.
                let user_token = Jwt::encode_user(
                    &conn,
                    &service,
                    user,
                    &key,
                    access_token_expires,
                    refresh_token_expires,
                )
                .map_err(GrpcMethodError::BadRequest)?;

                // Optionally create custom audit log.
                if let Some(x) = &req.audit {
                    let audit = audit
                        .create(driver, x, None, None)
                        .map_err(GrpcMethodError::BadRequest)?;
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
        audit: pb::uuid_opt_to_string_opt(audit.map(|x| x.id)),
    })
}

pub async fn revoke(
    server: &GrpcServer,
    request: GrpcMethodRequest<pb::AuthTokenRequest>,
) -> GrpcMethodResult<pb::AuthAuditReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthTokenRevoke,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                // Unsafely decode token to get user identifier, used to read key for safe token decode.
                let (user_id, token_type) = Jwt::decode_unsafe_user(&req.token, service.id)
                    .map_err(GrpcMethodError::BadRequest)?;

                // Token revoke requires token key type.
                // Do not check user, key is enabled or not revoked.
                let user = pattern::user_read_id_unchecked(driver, Some(&service), audit, user_id)
                    .map_err(GrpcMethodError::BadRequest)?;
                let key = pattern::key_read_user_unchecked(
                    driver,
                    &service,
                    audit,
                    &user,
                    KeyType::Token,
                )
                .map_err(GrpcMethodError::BadRequest)?;

                // Safely decode token with user key.
                let conn = driver.conn().map_err(GrpcMethodError::BadRequest)?;
                Jwt::decode_csrf(&conn, &service, &user, &key, token_type, &req.token)
                    .map_err(GrpcMethodError::BadRequest)?;

                // Token revoked, disable and revoke linked key.
                driver
                    .key_update(&KeyUpdate {
                        id: key.id,
                        is_enabled: Some(false),
                        is_revoked: Some(true),
                        name: None,
                    })
                    .map_err(GrpcMethodError::BadRequest)?;

                // Optionally create custom audit log.
                if let Some(x) = &req.audit {
                    let audit = audit
                        .create(driver, x, None, None)
                        .map_err(GrpcMethodError::BadRequest)?;
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
        audit: pb::uuid_opt_to_string_opt(audit.map(|x| x.id)),
    })
}
