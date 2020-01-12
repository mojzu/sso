use crate::grpc::{validate, Server};
use crate::{
    api::{self, ApiError},
    grpc::{methods::auth::api_csrf_verify, pb, util::*},
    *,
};
use tonic::{Response, Status};
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
) -> Result<Response<pb::AuthTokenVerifyReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthTokenVerify);
        let res: Result<(User, UserTokenAccess, Option<Audit>), Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;

            // Unsafely decode token to get user identifier, used to read key for safe token decode.
            let (user_id, _) =
                Jwt::decode_unsafe(&req.token, service.id).map_err(ApiError::BadRequest)?;

            // Token verify requires token key type.
            let user = pattern::user_read_id_checked(
                driver.as_ref().as_ref(),
                Some(&service),
                &mut audit,
                user_id,
            )
            .map_err(ApiError::BadRequest)?;
            let key = pattern::key_read_user_checked(
                driver.as_ref().as_ref(),
                &service,
                &mut audit,
                &user,
                KeyType::Token,
            )
            .map_err(ApiError::BadRequest)?;

            // Safely decode token with user key.
            let access_token_expires = Jwt::decode_access_token(&service, &user, &key, &req.token)
                .map_err(ApiError::BadRequest)?;

            // Token verified.
            let user_token = UserTokenAccess {
                user: user.clone(),
                access_token: req.token,
                access_token_expires,
            };

            // Optionally create custom audit log.
            if let Some(x) = req.audit {
                let audit = audit
                    .create(driver.as_ref().as_ref(), x, None, None)
                    .map_err(ApiError::BadRequest)?;
                Ok((user, user_token, Some(audit)))
            } else {
                Ok((user, user_token, None))
            }
        };
        let (user, token, audit) = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthTokenVerifyReply {
            user: Some(user.into()),
            access: Some(token.into()),
            audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn refresh(
    server: &Server,
    request: MethodRequest<pb::AuthTokenRequest>,
) -> Result<Response<pb::AuthTokenReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let access_token_expires = server.options().access_token_expires();
    let refresh_token_expires = server.options().refresh_token_expires();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthTokenRefresh);
        let res: Result<(UserToken, Option<Audit>), Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;

            // Unsafely decode token to get user identifier, used to read key for safe token decode.
            let (user_id, _) =
                Jwt::decode_unsafe(&req.token, service.id).map_err(ApiError::BadRequest)?;

            // Token refresh requires token key type.
            let user = pattern::user_read_id_checked(
                driver.as_ref().as_ref(),
                Some(&service),
                &mut audit,
                user_id,
            )
            .map_err(ApiError::BadRequest)?;
            let key = pattern::key_read_user_checked(
                driver.as_ref().as_ref(),
                &service,
                &mut audit,
                &user,
                KeyType::Token,
            )
            .map_err(ApiError::BadRequest)?;

            // Safely decode token with user key.
            let csrf_key = Jwt::decode_refresh_token(&service, &user, &key, &req.token)
                .map_err(ApiError::BadRequest)?;

            // Verify CSRF to prevent reuse.
            api_csrf_verify(driver.as_ref().as_ref(), &service, &csrf_key)?;

            // Encode user token.
            let user_token = Jwt::encode_user_token(
                driver.as_ref().as_ref(),
                &service,
                user,
                &key,
                access_token_expires,
                refresh_token_expires,
            )
            .map_err(ApiError::BadRequest)?;

            // Optionally create custom audit log.
            if let Some(x) = req.audit {
                let audit = audit
                    .create(driver.as_ref().as_ref(), x, None, None)
                    .map_err(ApiError::BadRequest)?;
                Ok((user_token, Some(audit)))
            } else {
                Ok((user_token, None))
            }
        };
        let (user_token, audit) = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthTokenReply {
            user: Some(user_token.user.clone().into()),
            access: Some(user_token.access_token()),
            refresh: Some(user_token.refresh_token()),
            audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn revoke(
    server: &Server,
    request: MethodRequest<pb::AuthTokenRequest>,
) -> Result<Response<pb::AuthAuditReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthTokenRevoke);
        let res: Result<Option<Audit>, Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;

            // Unsafely decode token to get user identifier, used to read key for safe token decode.
            let (user_id, token_type) =
                Jwt::decode_unsafe(&req.token, service.id).map_err(ApiError::BadRequest)?;

            // Token revoke requires token key type.
            // Do not check user, key is enabled or not revoked.
            let user = pattern::user_read_id_unchecked(
                driver.as_ref().as_ref(),
                Some(&service),
                &mut audit,
                user_id,
            )
            .map_err(ApiError::BadRequest)?;
            let key = pattern::key_read_user_unchecked(
                driver.as_ref().as_ref(),
                &service,
                &mut audit,
                &user,
                KeyType::Token,
            )
            .map_err(ApiError::BadRequest)?;

            // Safely decode token with user key.
            let csrf_key = Jwt::decode_csrf_key(&service, &user, &key, token_type, &req.token)
                .map_err(ApiError::BadRequest)?;
            if let Some(csrf_key) = csrf_key {
                driver.csrf_read(&csrf_key).map_err(ApiError::BadRequest)?;
            }

            // Token revoked, disable and revoked linked key.
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
