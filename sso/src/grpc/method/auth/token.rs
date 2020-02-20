use crate::{
    grpc::{pb, util::*, validate, Server},
    jwt, *,
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
                let (user_id, _) = jwt::decode_unsafe_user(&req.token, service.id)
                    .map_err(MethodError::BadRequest)?;

                // Token verify requires token key type.
                let user = pattern::user_read_id_checked(driver, Some(&service), audit, user_id)
                    .map_err(MethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
                        .map_err(MethodError::BadRequest)?;

                // Safely decode token with user key.
                let access_token_expires = jwt::decode_access(&service, &user, &key, &req.token)
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
                let (user_id, _) = jwt::decode_unsafe_user(&req.token, service.id)
                    .map_err(MethodError::BadRequest)?;

                // Token refresh requires token key type.
                let user = pattern::user_read_id_checked(driver, Some(&service), audit, user_id)
                    .map_err(MethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
                        .map_err(MethodError::BadRequest)?;

                // Safely decode token with user key.
                jwt::decode_refresh(driver, &service, &user, &key, &req.token)
                    .map_err(MethodError::BadRequest)?;

                // Encode user token.
                let user_token = jwt::encode_user(
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
                let (user_id, token_type) = jwt::decode_unsafe_user(&req.token, service.id)
                    .map_err(MethodError::BadRequest)?;

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
                jwt::decode_csrf(driver, &service, &user, &key, token_type, &req.token)
                    .map_err(MethodError::BadRequest)?;

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
