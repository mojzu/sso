pub mod github;
pub mod key;
pub mod local;
pub mod microsoft;
pub mod token;

use crate::grpc::{validate, Server};
use crate::{
    api::{self, ApiError, ApiResult},
    grpc::{pb, util::*},
    *,
};
use tonic::{Response, Status};
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
) -> Result<Response<pb::AuthAuditReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthTotp);
        let res: Result<(), Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;
            // TOTP requires token key type.
            let user = pattern::user_read_id_checked(
                driver.as_ref().as_ref(),
                Some(&service),
                &mut audit,
                string_to_uuid(req.user_id)?,
            )
            .map_err(ApiError::BadRequest)?;
            let key = pattern::key_read_user_checked(
                driver.as_ref().as_ref(),
                &service,
                &mut audit,
                &user,
                KeyType::Totp,
            )
            .map_err(ApiError::BadRequest)?;
            // Verify TOTP code.
            pattern::totp_verify(&key.value, &req.totp)
                .map_err(ApiError::BadRequest)
                .map_err::<tonic::Status, _>(Into::into)
        };
        api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthAuditReply { audit: None };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
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
) -> Result<Response<pb::AuthCsrfCreateReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthCsrfCreate);
        let res: Result<Csrf, Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;

            let expires_s = req.expires_s.unwrap_or(DEFAULT_CSRF_EXPIRES_S);
            driver
                .csrf_create(&CsrfCreate::generate(expires_s, service.id))
                .map_err(ApiError::BadRequest)
                .map_err::<tonic::Status, _>(Into::into)
        };
        let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthCsrfCreateReply {
            csrf: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
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
) -> Result<Response<pb::AuthAuditReply>, Status> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, Status, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthCsrfVerify);
        let res: Result<(), Status> = {
            let service =
                pattern::key_service_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;

            api_csrf_verify(driver.as_ref().as_ref(), &service, &req.csrf)
        };
        api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::AuthAuditReply { audit: None };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

// TODO(refactor): Improve structure.
fn api_csrf_verify(driver: &dyn Driver, service: &Service, csrf_key: &str) -> ApiResult<()> {
    driver
        .csrf_read(&csrf_key)
        .map_err(ApiError::BadRequest)
        .map_err::<Status, _>(Into::into)?
        .ok_or_else(|| DriverError::CsrfNotFoundOrUsed)
        .and_then(|csrf| csrf.check_service(service.id))
        .map_err(ApiError::BadRequest)
        .map_err::<Status, _>(Into::into)
}

fn oauth2_login(
    driver: &dyn Driver,
    audit: &mut AuditBuilder,
    service: &Service,
    service_id: Uuid,
    email: String,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> ApiResult<UserToken> {
    // Check service making url and callback requests match.
    if service.id != service_id {
        let e: tonic::Status = ApiError::BadRequest(DriverError::CsrfServiceMismatch).into();
        return Err(e);
    }

    // OAuth2 login requires token key type.
    let user = pattern::user_read_email_checked(driver, Some(&service), audit, email)
        .map_err(ApiError::BadRequest)?;
    let key = pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
        .map_err(ApiError::BadRequest)?;

    // Encode user token.
    Jwt::encode_user_token(
        driver,
        &service,
        user,
        &key,
        access_token_expires,
        refresh_token_expires,
    )
    .map_err(ApiError::BadRequest)
    .map_err::<tonic::Status, _>(Into::into)
}
