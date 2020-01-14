use crate::{
    grpc::{pb, util::*, validate, Server},
    *,
};
use tonic::Response;
use validator::{Validate, ValidationErrors};

impl Validate for pb::AuthKeyRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::key(e, "key", &self.key);
            validate::audit_type_opt(e, "audit", self.audit.as_ref().map(|x| &**x));
        })
    }
}

pub async fn verify(
    server: &Server,
    request: MethodRequest<pb::AuthKeyRequest>,
) -> MethodResponse<pb::AuthKeyReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let (user, key, audit) = audit_result_err(
            driver.as_ref().as_ref(),
            audit_meta,
            AuditType::AuthKeyVerify,
            |driver, audit| {
                let service = pattern::key_service_authenticate2(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                // Key verify requires key key type.
                let key = pattern::key_read_user_value_checked(
                    driver,
                    &service,
                    audit,
                    &req.key,
                    KeyType::Key,
                )
                .map_err(MethodError::BadRequest)?;
                let user = pattern::user_read_id_checked(
                    driver,
                    Some(&service),
                    audit,
                    key.user_id.unwrap(),
                )
                .map_err(MethodError::BadRequest)?;

                // Key verified.
                // Optionally create custom audit log.
                if let Some(x) = &req.audit {
                    let audit = audit
                        .create(driver, x, None, None)
                        .map_err(MethodError::BadRequest)?;
                    Ok((user, key, Some(audit)))
                } else {
                    Ok((user, key, None))
                }
            },
        )?;
        let reply = pb::AuthKeyReply {
            user: Some(user.into()),
            key: Some(key.into()),
            audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn revoke(
    server: &Server,
    request: MethodRequest<pb::AuthKeyRequest>,
) -> MethodResponse<pb::AuthAuditReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthKeyRevoke);

        let blocking_inner = || {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(MethodError::Unauthorised)?;

            // Key revoke requires key key type.
            // Do not check key is enabled or not revoked.
            let key = pattern::key_read_user_value_unchecked(
                driver.as_ref().as_ref(),
                &service,
                &mut audit,
                req.key,
                KeyType::Key,
            )
            .map_err(MethodError::BadRequest)?;

            // Disable and revoke key.
            driver
                .key_update(&KeyUpdate {
                    id: key.id,
                    is_enabled: Some(false),
                    is_revoked: Some(true),
                    name: None,
                })
                .map_err(MethodError::BadRequest)?;

            // Optionally create custom audit log.
            if let Some(x) = req.audit {
                let audit = audit
                    .create(driver.as_ref().as_ref(), x, None, None)
                    .map_err(MethodError::BadRequest)?;
                Ok(Some(audit))
            } else {
                Ok(None)
            }
        };
        let res: Result<Option<Audit>, MethodError> = blocking_inner();
        let audit = audit_result(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthAuditReply {
            audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}
