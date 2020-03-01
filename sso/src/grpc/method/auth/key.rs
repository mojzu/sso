use crate::prelude::*;

impl validator::Validate for pb::AuthKeyRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        validate::wrap(|e| {
            validate::key(e, "key", &self.key);
            validate::audit_type_opt(e, "audit", self.audit.as_ref().map(|x| &**x));
        })
    }
}

pub async fn verify(
    server: &GrpcServer,
    request: GrpcMethodRequest<pb::AuthKeyRequest>,
) -> GrpcMethodResult<pb::AuthKeyReply> {
    let (audit_meta, auth, req) = request.into_inner();
    let driver = server.driver();

    blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthKeyVerify,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                // Key verify requires key key type.
                let key = pattern::key_read_user_value_checked(
                    driver,
                    &service,
                    audit,
                    &req.key,
                    KeyType::Key,
                )
                .map_err(GrpcMethodError::BadRequest)?;
                let user = pattern::user_read_id_checked(
                    driver,
                    Some(&service),
                    audit,
                    key.user_id.unwrap(),
                )
                .map_err(GrpcMethodError::BadRequest)?;

                // Key verified.
                // Optionally create custom audit log.
                if let Some(x) = &req.audit {
                    let audit = audit
                        .create(driver, x, None, None)
                        .map_err(GrpcMethodError::BadRequest)?;
                    Ok((user, key, Some(audit)))
                } else {
                    Ok((user, key, None))
                }
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|(user, key, audit)| pb::AuthKeyReply {
        user: Some(user.into()),
        key: Some(key.into()),
        audit: pb::uuid_opt_to_string_opt(audit.map(|x| x.id)),
    })
}

pub async fn revoke(
    server: &GrpcServer,
    request: GrpcMethodRequest<pb::AuthKeyRequest>,
) -> GrpcMethodResult<pb::AuthAuditReply> {
    let (audit_meta, auth, req) = request.into_inner();
    let driver = server.driver();

    blocking_method(move || {
        audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthKeyRevoke,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                // Key revoke requires key key type.
                // Do not check key is enabled or not revoked.
                let key = pattern::key_read_user_value_unchecked(
                    driver,
                    &service,
                    audit,
                    &req.key,
                    KeyType::Key,
                )
                .map_err(GrpcMethodError::BadRequest)?;

                // Disable and revoke key.
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
