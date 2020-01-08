use crate::grpc::{validate, Server};
use crate::{
    api::{self, ApiError},
    grpc::{pb, util::*},
    *,
};
use tonic::{Response, Status};
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
    request: MetaRequest<pb::AuthKeyRequest>,
) -> Result<Response<pb::AuthKeyReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthKeyVerify);
        let res: Result<(User, KeyWithValue, Option<Audit>), Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;

            // Key verify requires key key type.
            let key = pattern::key_read_user_value_checked(
                driver.as_ref().as_ref(),
                &service,
                &mut audit,
                req.key,
                KeyType::Key,
            )
            .map_err(ApiError::BadRequest)?;
            let user = pattern::user_read_id_checked(
                driver.as_ref().as_ref(),
                Some(&service),
                &mut audit,
                key.user_id.unwrap(),
            )
            .map_err(ApiError::BadRequest)?;

            // Key verified.
            // Optionally create custom audit log.
            if let Some(x) = req.audit {
                let audit = audit
                    .create(driver.as_ref().as_ref(), x, None, None)
                    .map_err(ApiError::BadRequest)?;
                Ok((user, key, Some(audit)))
            } else {
                Ok((user, key, None))
            }
        };
        let (user, key, audit) = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
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
    request: MetaRequest<pb::AuthKeyRequest>,
) -> Result<Response<pb::AuthAuditReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthKeyRevoke);
        let res: Result<Option<Audit>, Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;

            // Key revoke requires key key type.
            // Do not check key is enabled or not revoked.
            let key = pattern::key_read_user_value_unchecked(
                driver.as_ref().as_ref(),
                &service,
                &mut audit,
                req.key,
                KeyType::Key,
            )
            .map_err(ApiError::BadRequest)?;

            // Disable and revoke key.
            driver
                .key_update(&KeyUpdate {
                    id: key.id,
                    is_enabled: Some(false),
                    is_revoked: Some(true),
                    name: None,
                })
                .map_err(ApiError::BadRequest)?;

            // Optionally create custom audit log.
            if let Some(x) = req.audit {
                let audit = audit
                    .create(driver.as_ref().as_ref(), x, None, None)
                    .map_err(ApiError::BadRequest)?;
                Ok(Some(audit))
            } else {
                Ok(None)
            }
        };
        let audit = api::result_audit(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthAuditReply {
            audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}
