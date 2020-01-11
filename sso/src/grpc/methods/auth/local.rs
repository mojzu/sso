use crate::grpc::{validate, Server};
use crate::{
    api::{self, ApiError, ApiResult},
    grpc::{methods::auth::api_csrf_verify, pb, util::*},
    *,
};
use tonic::{Response, Status};
use validator::{Validate, ValidationErrors};

impl Validate for pb::AuthLoginRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::email(e, "email", &self.email);
            validate::password(e, "password", &self.password);
        })
    }
}

pub async fn login(
    server: &Server,
    request: MetaRequest<pb::AuthLoginRequest>,
) -> Result<Response<pb::AuthLoginReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let client = server.client();
    let password_pwned_enabled = server.options().password_pwned_enabled();
    let access_token_expires = server.options().access_token_expires();
    let refresh_token_expires = server.options().refresh_token_expires();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalLogin);
        let password_meta = api::password_meta(
            client.as_ref(),
            password_pwned_enabled,
            Some(req.password.clone()),
        )
        .map_err(ApiError::BadRequest)?;

        let res: Result<UserToken, Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;

            // Login requires token key type.
            let user = pattern::user_read_email_checked(
                driver.as_ref().as_ref(),
                Some(&service),
                &mut audit,
                req.email,
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

            // Forbidden if user password update required.
            if user.password_require_update {
                let e: Status = ApiError::Forbidden(DriverError::UserPasswordUpdateRequired).into();
                return Err(e);
            }

            // Check user password.
            user.password_check(&req.password)
                .map_err(ApiError::BadRequest)?;

            // Encode user token.
            Jwt::encode_user_token(
                driver.as_ref().as_ref(),
                &service,
                user,
                &key,
                access_token_expires,
                refresh_token_expires,
            )
            .map_err(ApiError::BadRequest)
            .map_err::<Status, _>(Into::into)
        };
        let user_token = api::result_audit(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthLoginReply {
            meta: Some(password_meta.into()),
            user: Some(user_token.user.clone().into()),
            access: Some(user_token.access_token()),
            refresh: Some(user_token.refresh_token()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::AuthRegisterRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::name(e, "name", &self.name);
            validate::email(e, "email", &self.email);
            validate::locale_opt(e, "locale", self.locale.as_ref().map(|x| &**x));
            validate::timezone_opt(e, "timezone", self.timezone.as_ref().map(|x| &**x));
        })
    }
}

pub async fn register(
    server: &Server,
    request: MetaRequest<pb::AuthRegisterRequest>,
) -> Result<Response<()>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let access_token_expires = server.options().access_token_expires();
    let email = server.smtp_email();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalRegister);
        let res: Result<(), Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;
            // Bad request if service not allowed to register users.
            if !service.user_allow_register {
                let e: tonic::Status =
                    ApiError::BadRequest(DriverError::ServiceUserRegisterDisabled).into();
                return Err(e);
            }
            // Create user, is allowed to request password reset in case register token expires.
            // TODO(refactor): Support user for email already exists, add test for this.
            let mut user_create =
                UserCreate::new(true, &req.name, req.email).password_allow_reset(true);
            if let Some(locale) = req.locale {
                user_create = user_create.locale(locale);
            }
            if let Some(timezone) = req.timezone {
                user_create = user_create.timezone(timezone);
            }
            let user = driver
                .user_create(&user_create)
                .map_err(ApiError::BadRequest)?;
            // Create token key for user.
            let key_create = KeyCreate::user(true, KeyType::Token, &req.name, service.id, user.id);
            let key = driver
                .key_create(&key_create)
                .map_err(ApiError::BadRequest)?;
            // Encode register token.
            let token = Jwt::encode_register_token(
                driver.as_ref().as_ref(),
                &service,
                &user,
                &key,
                access_token_expires,
            )
            .map_err(ApiError::BadRequest)?;
            // Send register email.
            let e = TemplateEmail::email_register(&service, &user, &token, audit.meta())
                .map_err(ApiError::BadRequest)?;
            email(e)
                .map_err::<DriverError, _>(Into::into)
                .map_err(ApiError::BadRequest)?;
            Ok(())
        };
        api::result_audit(driver.as_ref().as_ref(), &audit, res)?;
        Ok(())
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::AuthRegisterConfirmRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::token(e, "token", &self.token);
            validate::password_opt(e, "password", self.password.as_ref().map(|x| &**x));
        })
    }
}

pub async fn register_confirm(
    server: &Server,
    request: MetaRequest<pb::AuthRegisterConfirmRequest>,
) -> Result<Response<pb::AuthPasswordMetaReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let client = server.client();
    let password_pwned_enabled = server.options().password_pwned_enabled();
    let revoke_token_expires = server.options().revoke_token_expires();
    let email = server.smtp_email();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalRegisterConfirm);
        let password_meta = api::password_meta(
            client.as_ref(),
            password_pwned_enabled,
            req.password.clone(),
        )
        .map_err(ApiError::BadRequest)?;

        let res: Result<(), Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;
            // Bad request if service not allowed to register users.
            if !service.user_allow_register {
                let e: tonic::Status =
                    ApiError::BadRequest(DriverError::ServiceUserRegisterDisabled).into();
                return Err(e);
            }
            // Unsafely decode token to get user identifier, used to read key for safe token decode.
            let (user_id, _) =
                Jwt::decode_unsafe(&req.token, service.id).map_err(ApiError::BadRequest)?;
            // Register confirm requires token key type.
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
            let csrf_key = Jwt::decode_register_token(&service, &user, &key, &req.token)
                .map_err(ApiError::BadRequest)?;
            // Verify CSRF to prevent reuse.
            api_csrf_verify(driver.as_ref().as_ref(), &service, &csrf_key)?;
            // Encode revoke token.
            let token = Jwt::encode_revoke_token(
                driver.as_ref().as_ref(),
                &service,
                &user,
                &key,
                revoke_token_expires,
            )
            .map_err(ApiError::BadRequest)?;
            // Update user password and allow reset flag if provided.
            if let Some(password) = req.password {
                let mut user_update =
                    UserUpdate::new_password(user.id, password).map_err(ApiError::BadRequest)?;
                if let Some(password_allow_reset) = req.password_allow_reset {
                    user_update = user_update.set_password_allow_reset(password_allow_reset);
                }
                driver
                    .user_update(&user_update)
                    .map_err(ApiError::BadRequest)?;
            }
            // Send reset password confirm email.
            let e = TemplateEmail::email_register_confirm(&service, &user, &token, audit.meta())
                .map_err(ApiError::BadRequest)?;
            email(e)
                .map_err::<DriverError, _>(Into::into)
                .map_err(ApiError::BadRequest)?;
            Ok(())
        };
        api::result_audit(driver.as_ref().as_ref(), &audit, res)?;
        Ok(pb::AuthPasswordMetaReply {
            meta: Some(password_meta.into()),
        })
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn register_revoke(
    server: &Server,
    request: MetaRequest<pb::AuthTokenRequest>,
) -> Result<Response<pb::AuthAuditReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalRegisterRevoke);
        let res: Result<Option<Audit>, Status> =
            { revoke_inner(driver.as_ref().as_ref(), &mut audit, auth, req) };
        let audit = api::result_audit(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthAuditReply {
            audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::AuthResetPasswordRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::email(e, "email", &self.email);
        })
    }
}

pub async fn reset_password(
    server: &Server,
    request: MetaRequest<pb::AuthResetPasswordRequest>,
) -> Result<Response<()>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let access_token_expires = server.options().access_token_expires();
    let email = server.smtp_email();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalResetPassword);
        let res: Result<(), Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;
            // Reset password requires token key type.
            let user = pattern::user_read_email_checked(
                driver.as_ref().as_ref(),
                Some(&service),
                &mut audit,
                req.email,
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
            // Bad request if user password reset is disabled.
            if !user.password_allow_reset {
                let e: tonic::Status =
                    ApiError::BadRequest(DriverError::UserResetPasswordDisabled).into();
                return Err(e);
            }
            // Encode reset token.
            let token = Jwt::encode_reset_password_token(
                driver.as_ref().as_ref(),
                &service,
                &user,
                &key,
                access_token_expires,
            )
            .map_err(ApiError::BadRequest)?;
            // Send reset password email.
            let e = TemplateEmail::email_reset_password(&service, &user, &token, audit.meta())
                .map_err(ApiError::BadRequest)?;
            email(e)
                .map_err::<DriverError, _>(Into::into)
                .map_err(ApiError::BadRequest)?;
            Ok(())
        };
        api::result_audit(driver.as_ref().as_ref(), &audit, res)?;
        // TODO(refactor): Fix this.
        // // Catch Err result so this function returns Ok to prevent the caller
        // // from inferring a users existence.
        // .or_else(|_e| Ok(()))?;
        Ok(())
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::AuthResetPasswordConfirmRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::token(e, "token", &self.token);
            validate::password(e, "password", &self.password);
        })
    }
}

pub async fn reset_password_confirm(
    server: &Server,
    request: MetaRequest<pb::AuthResetPasswordConfirmRequest>,
) -> Result<Response<pb::AuthPasswordMetaReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let client = server.client();
    let password_pwned_enabled = server.options().password_pwned_enabled();
    let revoke_token_expires = server.options().revoke_token_expires();
    let email = server.smtp_email();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalResetPasswordConfirm);
        let password_meta = api::password_meta(
            client.as_ref(),
            password_pwned_enabled,
            Some(req.password.clone()),
        )
        .map_err(ApiError::BadRequest)?;

        let res: Result<(), Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;

            // Unsafely decode token to get user identifier, used to read key for safe token decode.
            let (user_id, _) =
                Jwt::decode_unsafe(&req.token, service.id).map_err(ApiError::BadRequest)?;

            // Reset password confirm requires token key type.
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

            // Bad request if user password reset is disabled.
            if !user.password_allow_reset {
                let e = ApiError::BadRequest(DriverError::UserResetPasswordDisabled).into();
                return Err(e);
            }

            // Safely decode token with user key.
            let csrf_key = Jwt::decode_reset_password_token(&service, &user, &key, &req.token)
                .map_err(ApiError::BadRequest)?;

            // Verify CSRF to prevent reuse.
            api_csrf_verify(driver.as_ref().as_ref(), &service, &csrf_key)?;

            // Encode revoke token.
            let token = Jwt::encode_revoke_token(
                driver.as_ref().as_ref(),
                &service,
                &user,
                &key,
                revoke_token_expires,
            )
            .map_err(ApiError::BadRequest)?;

            // Update user password.
            let user_update =
                UserUpdate::new_password(user.id, req.password).map_err(ApiError::BadRequest)?;
            driver
                .user_update(&user_update)
                .map_err(ApiError::BadRequest)?;

            // Send reset password confirm email.
            let e =
                TemplateEmail::email_reset_password_confirm(&service, &user, &token, audit.meta())
                    .map_err(ApiError::BadRequest)?;
            email(e)
                .map_err::<DriverError, _>(Into::into)
                .map_err(ApiError::BadRequest)?;
            Ok(())
        };
        api::result_audit(driver.as_ref().as_ref(), &audit, res)?;
        Ok(pb::AuthPasswordMetaReply {
            meta: Some(password_meta.into()),
        })
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn reset_password_revoke(
    server: &Server,
    request: MetaRequest<pb::AuthTokenRequest>,
) -> Result<Response<pb::AuthAuditReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalResetPasswordRevoke);
        let res: Result<Option<Audit>, Status> =
            { revoke_inner(driver.as_ref().as_ref(), &mut audit, auth, req) };
        let audit = api::result_audit(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthAuditReply {
            audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::AuthUpdateEmailRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "user_id", &self.user_id);
            validate::password(e, "password", &self.password);
            validate::email(e, "new_email", &self.new_email);
        })
    }
}

pub async fn update_email(
    server: &Server,
    request: MetaRequest<pb::AuthUpdateEmailRequest>,
) -> Result<Response<()>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let revoke_token_expires = server.options().revoke_token_expires();
    let email = server.smtp_email();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalUpdateEmail);
        let res: Result<(), Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;
            // Update email requires token key type.
            let user = pattern::user_read_id_checked(
                driver.as_ref().as_ref(),
                Some(&service),
                &mut audit,
                string_to_uuid(req.user_id.clone())?,
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
            // Forbidden if user password update required.
            if user.password_require_update {
                let e: tonic::Status =
                    ApiError::Forbidden(DriverError::UserPasswordUpdateRequired).into();
                return Err(e);
            }
            // Check user password.
            user.password_check(&req.password)
                .map_err(ApiError::BadRequest)?;
            // Encode revoke token.
            let token = Jwt::encode_revoke_token(
                driver.as_ref().as_ref(),
                &service,
                &user,
                &key,
                revoke_token_expires,
            )
            .map_err(ApiError::BadRequest)?;
            // Update user email.
            let old_email = user.email.to_owned();
            driver
                .user_update(&UserUpdate::new_email(user.id, req.new_email))
                .map_err(ApiError::BadRequest)?;
            let user = pattern::user_read_id_checked(
                driver.as_ref().as_ref(),
                Some(&service),
                &mut audit,
                string_to_uuid(req.user_id)?,
            )
            .map_err(ApiError::BadRequest)?;
            // Send update email email.
            let e = TemplateEmail::email_update_email(
                &service,
                &user,
                &old_email,
                &token,
                audit.meta(),
            )
            .map_err(ApiError::BadRequest)?;
            email(e)
                .map_err::<DriverError, _>(Into::into)
                .map_err(ApiError::BadRequest)?;
            Ok(())
        };
        api::result_audit(driver.as_ref().as_ref(), &audit, res)?;
        Ok(())
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn update_email_revoke(
    server: &Server,
    request: MetaRequest<pb::AuthTokenRequest>,
) -> Result<Response<pb::AuthAuditReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalUpdateEmailRevoke);
        let res: Result<Option<Audit>, Status> =
            { revoke_inner(driver.as_ref().as_ref(), &mut audit, auth, req) };
        let audit = api::result_audit(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthAuditReply {
            audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::AuthUpdatePasswordRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "user_id", &self.user_id);
            validate::password(e, "password", &self.password);
            validate::password(e, "new_password", &self.new_password);
        })
    }
}

pub async fn update_password(
    server: &Server,
    request: MetaRequest<pb::AuthUpdatePasswordRequest>,
) -> Result<Response<pb::AuthPasswordMetaReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let client = server.client();
    let password_pwned_enabled = server.options().password_pwned_enabled();
    let revoke_token_expires = server.options().revoke_token_expires();
    let email = server.smtp_email();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalUpdatePassword);
        let password_meta = api::password_meta(
            client.as_ref(),
            password_pwned_enabled,
            Some(req.password.clone()),
        )
        .map_err(ApiError::BadRequest)?;

        let res: Result<(), Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;
            // Update password requires token key type.
            let user = pattern::user_read_id_checked(
                driver.as_ref().as_ref(),
                Some(&service),
                &mut audit,
                string_to_uuid(req.user_id.clone())?,
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
            // User is allowed to update password if `password_require_update` is true.
            // Check user password.
            user.password_check(&req.password)
                .map_err(ApiError::BadRequest)?;
            // Encode revoke token.
            let token = Jwt::encode_revoke_token(
                driver.as_ref().as_ref(),
                &service,
                &user,
                &key,
                revoke_token_expires,
            )
            .map_err(ApiError::BadRequest)?;
            // Update user password.
            let user_update = UserUpdate::new_password(user.id, req.new_password)
                .map_err(ApiError::BadRequest)?;
            driver
                .user_update(&user_update)
                .map_err(ApiError::BadRequest)?;
            let user = pattern::user_read_id_checked(
                driver.as_ref().as_ref(),
                Some(&service),
                &mut audit,
                string_to_uuid(req.user_id)?,
            )
            .map_err(ApiError::BadRequest)?;
            // Send update password email.
            let e = TemplateEmail::email_update_password(&service, &user, &token, audit.meta())
                .map_err(ApiError::BadRequest)?;
            email(e)
                .map_err::<DriverError, _>(Into::into)
                .map_err(ApiError::BadRequest)?;
            Ok(())
        };
        api::result_audit(driver.as_ref().as_ref(), &audit, res)?;
        Ok(pb::AuthPasswordMetaReply {
            meta: Some(password_meta.into()),
        })
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn update_password_revoke(
    server: &Server,
    request: MetaRequest<pb::AuthTokenRequest>,
) -> Result<Response<pb::AuthAuditReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalUpdatePasswordRevoke);
        let res: Result<Option<Audit>, Status> =
            { revoke_inner(driver.as_ref().as_ref(), &mut audit, auth, req) };
        let audit = api::result_audit(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthAuditReply {
            audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

fn revoke_inner(
    driver: &dyn Driver,
    audit: &mut AuditBuilder,
    auth: Option<String>,
    req: pb::AuthTokenRequest,
) -> ApiResult<Option<Audit>> {
    let service =
        pattern::key_service_authenticate(driver, audit, auth).map_err(ApiError::Unauthorised)?;
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = Jwt::decode_unsafe(&req.token, service.id).map_err(ApiError::BadRequest)?;
    // Update email revoke requires token key type.
    // Do not check user, key is enabled or not revoked.
    let user = pattern::user_read_id_unchecked(driver, Some(&service), audit, user_id)
        .map_err(ApiError::BadRequest)?;
    let key = pattern::key_read_user_unchecked(driver, &service, audit, &user, KeyType::Token)
        .map_err(ApiError::BadRequest)?;
    // Safely decode token with user key.
    let csrf_key = Jwt::decode_revoke_token(&service, &user, &key, &req.token)
        .map_err(ApiError::BadRequest)?;
    // Verify CSRF to prevent reuse.
    api_csrf_verify(driver, &service, &csrf_key)?;
    // TODO(refactor): Rethink this behaviour?
    // Disable user and disable and revoke all keys associated with user.
    // driver
    //     .user_update(&user.id, &UserUpdate::default().set_is_enabled(false))
    //     .map_err(ApiError::BadRequest)?;
    // driver
    //     .key_update_many(
    //         &user.id,
    //         &KeyUpdate {
    //             is_enabled: Some(false),
    //             is_revoked: Some(true),
    //             name: None,
    //         },
    //     )
    //     .map_err(ApiError::BadRequest)?;
    // Optionally create custom audit log.
    if let Some(x) = req.audit {
        let audit = audit
            .create(driver, x, None, None)
            .map_err(ApiError::BadRequest)?;
        Ok(Some(audit))
    } else {
        Ok(None)
    }
}
