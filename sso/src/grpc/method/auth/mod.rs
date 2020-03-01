pub mod github;
pub mod key;
pub mod local;
pub mod microsoft;
pub mod token;

use crate::prelude::*;

impl validator::Validate for pb::AuthTotpRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "user_id", &self.user_id);
            validate::totp(e, "totp", &self.totp);
        })
    }
}

pub async fn totp_verify(
    server: &GrpcServer,
    request: GrpcMethodRequest<pb::AuthTotpRequest>,
) -> GrpcMethodResult<pb::AuthAuditReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthTotp,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;
                // TOTP requires token key type.
                let user = pattern::user_read_id_checked(
                    driver,
                    Some(&service),
                    audit,
                    pb::string_to_uuid(req.user_id.clone()),
                )
                .map_err(GrpcMethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Totp)
                        .map_err(GrpcMethodError::BadRequest)?;
                // Verify TOTP code.
                pattern::totp_verify(&key.value, &req.totp).map_err(GrpcMethodError::BadRequest)
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|_data| pb::AuthAuditReply { audit: None })
}

pub async fn csrf_create(
    server: &GrpcServer,
    request: GrpcMethodRequest<pb::AuthCsrfCreateRequest>,
) -> GrpcMethodResult<pb::AuthCsrfCreateReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthCsrfCreate,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                let conn = driver.conn().map_err(GrpcMethodError::BadRequest)?;
                CsrfCreate::request(&conn, &req, service.id).map_err(GrpcMethodError::BadRequest)
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|data| pb::AuthCsrfCreateReply { csrf: Some(data) })
}

pub async fn csrf_verify(
    server: &GrpcServer,
    request: GrpcMethodRequest<pb::AuthCsrfVerifyRequest>,
) -> GrpcMethodResult<pb::AuthAuditReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthCsrfVerify,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;

                let conn = driver.conn().map_err(GrpcMethodError::BadRequest)?;
                CsrfVerify::request(&conn, &req, service.id).map_err(GrpcMethodError::BadRequest)
            },
        )
        .map_err(Into::into)
    })
    .await
    .map(|_| pb::AuthAuditReply { audit: None })
}

fn oauth2_login(
    driver: &Postgres,
    audit: &mut AuditBuilder,
    service: &Service,
    service_id: Uuid,
    email: String,
    access_token_expires: Duration,
    refresh_token_expires: Duration,
) -> GrpcMethodResult<UserToken> {
    // Check service making url and callback requests match.
    if service.id != service_id {
        return Err(GrpcMethodError::BadRequest(
            DriverError::CsrfServiceMismatch,
        ));
    }

    // OAuth2 login requires token key type.
    let user = pattern::user_read_email_checked(driver, Some(&service), audit, &email)
        .map_err(GrpcMethodError::BadRequest)?;
    let key = pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
        .map_err(GrpcMethodError::BadRequest)?;

    // Encode user token.
    let conn = driver.conn().map_err(GrpcMethodError::BadRequest)?;
    Jwt::encode_user(
        &conn,
        &service,
        user,
        &key,
        access_token_expires,
        refresh_token_expires,
    )
    .map_err(GrpcMethodError::BadRequest)
}
