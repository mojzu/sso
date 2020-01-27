pub mod github;
pub mod key;
pub mod local;
pub mod microsoft;
pub mod token;

use crate::{
    grpc::{pb, util::*, validate, Server},
    *,
};
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

impl Validate for pb::AuthTotpRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "user_id", &self.user_id);
            validate::totp(e, "totp", &self.totp);
        })
    }
}

pub async fn totp_verify(
    server: &Server,
    request: MethodRequest<pb::AuthTotpRequest>,
) -> MethodResult<pb::AuthAuditReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking::<_, MethodError, _>(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthTotp,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;
                // TOTP requires token key type.
                let user = pattern::user_read_id_checked(
                    driver,
                    Some(&service),
                    audit,
                    string_to_uuid(req.user_id.clone()),
                )
                .map_err(MethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Totp)
                        .map_err(MethodError::BadRequest)?;
                // Verify TOTP code.
                pattern::totp_verify(&key.value, &req.totp).map_err(MethodError::BadRequest)
            },
        )?;
        let reply = pb::AuthAuditReply { audit: None };
        Ok(reply)
    })
    .await
}

impl Validate for pb::AuthCsrfCreateRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::csrf_expires_s_opt(e, "expires_s", self.expires_s);
        })
    }
}

pub async fn csrf_create(
    server: &Server,
    request: MethodRequest<pb::AuthCsrfCreateRequest>,
) -> MethodResult<pb::AuthCsrfCreateReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking::<_, MethodError, _>(move || {
        let data = audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthCsrfCreate,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                let expires_s = req.expires_s.unwrap_or(DEFAULT_CSRF_EXPIRES_S);
                driver
                    .csrf_create(&CsrfCreate::generate(expires_s, service.id))
                    .map_err(MethodError::BadRequest)
            },
        )?;
        let reply = pb::AuthCsrfCreateReply {
            csrf: Some(data.into()),
        };
        Ok(reply)
    })
    .await
}

impl Validate for pb::AuthCsrfVerifyRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::csrf_token(e, "csrf", &self.csrf);
            validate::audit_type_opt(e, "audit", self.audit.as_ref().map(|x| &**x))
        })
    }
}

pub async fn csrf_verify(
    server: &Server,
    request: MethodRequest<pb::AuthCsrfVerifyRequest>,
) -> MethodResult<pb::AuthAuditReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    blocking::<_, MethodError, _>(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthCsrfVerify,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                api_csrf_verify(driver, &service, &req.csrf)
            },
        )?;
        let reply = pb::AuthAuditReply { audit: None };
        Ok(reply)
    })
    .await
}

// TODO(3,refactor): Improve code structure.
fn api_csrf_verify(driver: &Postgres, service: &Service, csrf_key: &str) -> MethodResult<Csrf> {
    driver
        .csrf_read(&csrf_key)
        .map_err(MethodError::BadRequest)?
        .ok_or_else(|| DriverError::CsrfNotFoundOrUsed)
        .and_then(|csrf| {
            csrf.check_service(service.id)?;
            Ok(csrf)
        })
        .map_err(MethodError::BadRequest)
}

fn oauth2_login(
    driver: &Postgres,
    audit: &mut AuditBuilder,
    service: &Service,
    service_id: Uuid,
    email: String,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> MethodResult<UserToken> {
    // Check service making url and callback requests match.
    if service.id != service_id {
        return Err(MethodError::BadRequest(DriverError::CsrfServiceMismatch));
    }

    // OAuth2 login requires token key type.
    let user = pattern::user_read_email_checked(driver, Some(&service), audit, &email)
        .map_err(MethodError::BadRequest)?;
    let key = pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
        .map_err(MethodError::BadRequest)?;

    // Encode user token.
    Jwt::encode_user_token(
        driver,
        &service,
        user,
        &key,
        access_token_expires,
        refresh_token_expires,
    )
    .map_err(MethodError::BadRequest)
}
