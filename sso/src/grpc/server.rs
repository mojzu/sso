use crate::{grpc::method, prelude::*};
use lettre::{file::FileTransport, SmtpClient, Transport};
use lettre_email::Email;
use prometheus::{HistogramTimer, HistogramVec, IntCounterVec};
use std::{fmt, path::PathBuf, sync::Arc};

/// gRPC server metrics.
pub struct GrpcServerMetrics {
    path: String,
    count: IntCounterVec,
    timer: HistogramTimer,
}

impl fmt::Debug for GrpcServerMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GrpcServerMetrics {{ path: {}, ... }}", self.path)
    }
}

impl GrpcServerMetrics {
    /// Start
    pub fn start<P: AsRef<str>>(path: P, count: &IntCounterVec, latency: &HistogramVec) -> Self {
        let timer = latency.with_label_values(&[path.as_ref()]).start_timer();
        Self {
            path: path.as_ref().to_owned(),
            count: count.clone(),
            timer,
        }
    }

    pub fn end(self, status: u16) {
        let status = format!("{}", status);
        self.timer.observe_duration();
        self.count
            .with_label_values(&[&self.path, &status])
            .inc_by(1);
    }
}

/// gRPC server.
#[derive(Clone)]
pub struct GrpcServer {
    options: GrpcServerOptions,
    driver: Arc<Postgres>,
    client: Arc<reqwest::Client>,
    smtp_client: Arc<Option<SmtpClient>>,
    count: IntCounterVec,
    latency: HistogramVec,
}

impl fmt::Debug for GrpcServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GrpcServer {{ ... }}")
    }
}

impl GrpcServer {
    /// Returns new server.
    pub fn new(driver: Postgres, options: GrpcServerOptions) -> Self {
        let client = options.client().unwrap();
        let smtp_client = options.smtp_client().unwrap();
        let (count, latency) = Metrics::grpc_metrics();
        Self {
            options,
            driver: Arc::new(driver),
            client: Arc::new(client),
            smtp_client: Arc::new(smtp_client),
            count,
            latency,
        }
    }

    /// Returns tonic service.
    pub fn service(self) -> pb::sso_server::SsoServer<Self> {
        pb::sso_server::SsoServer::new(self)
    }

    /// Returns reference to `GrpcServerOptions`.
    pub(crate) fn options(&self) -> &GrpcServerOptions {
        &self.options
    }

    /// Returns reference to driver.
    pub fn driver(&self) -> Arc<Postgres> {
        self.driver.clone()
    }

    /// Returns reference to HTTP client.
    pub(crate) fn client(&self) -> Arc<reqwest::Client> {
        self.client.clone()
    }

    /// Build email callback function. Must be called from blocking context.
    /// If client is None and file directory path is provided, file transport is used.
    pub(crate) fn smtp_email(&self) -> Box<dyn FnOnce(TemplateEmail) -> DriverResult<()> + Send> {
        let client = self.smtp_client.clone();
        let from_email = self.options.smtp_from_email();
        let smtp_file = self.options.smtp_file();

        Box::new(move |email| {
            let email_builder = Email::builder()
                .to((email.to_email, email.to_name))
                .subject(email.subject)
                .text(email.text);

            match (client.as_ref(), smtp_file) {
                (Some(client), _) => {
                    let email = email_builder
                        .from((from_email.unwrap(), email.from_name))
                        .build()
                        .map_err(DriverError::LettreEmail)?;

                    let mut transport = client.clone().transport();
                    transport.send(email.into()).map_err(DriverError::Lettre)?;
                    Ok(())
                }
                (_, Some(smtp_file)) => {
                    let email = email_builder
                        .from(("file@localhost", email.from_name))
                        .build()
                        .map_err(DriverError::LettreEmail)?;

                    let path = PathBuf::from(smtp_file);
                    let mut transport = FileTransport::new(path);
                    transport
                        .send(email.into())
                        .map_err(DriverError::LettreFile)?;
                    Ok(())
                }
                (None, None) => Err(DriverError::SmtpDisabled),
            }
        })
    }

    fn pre(
        &self,
        path: &str,
        req: tonic::Request<()>,
    ) -> Result<(GrpcServerMetrics, GrpcMethodRequest<()>), tonic::Status> {
        let metrics = GrpcServerMetrics::start(path, &self.count, &self.latency);
        Ok((
            metrics,
            GrpcMethodRequest::from_unit(req, self.options().traefik_enabled())?,
        ))
    }

    fn pre_validate<R, T>(
        &self,
        path: &str,
        req: tonic::Request<R>,
    ) -> Result<(GrpcServerMetrics, GrpcMethodRequest<T>), tonic::Status>
    where
        R: validator::Validate,
        T: From<R>,
    {
        let metrics = GrpcServerMetrics::start(path, &self.count, &self.latency);
        Ok((
            metrics,
            GrpcMethodRequest::from_request(req, self.options().traefik_enabled())?,
        ))
    }

    fn post<T, E>(
        &self,
        metrics: GrpcServerMetrics,
        res: Result<T, E>,
    ) -> Result<tonic::Response<T>, tonic::Status>
    where
        E: Into<tonic::Status>,
    {
        match res {
            Ok(x) => {
                metrics.end(0);
                Ok(tonic::Response::new(x))
            }
            Err(e) => {
                let e: tonic::Status = e.into();
                metrics.end(e.code() as u16);
                Err(e)
            }
        }
    }
}

#[tonic::async_trait]
impl pb::sso_server::Sso for GrpcServer {
    async fn ping(&self, _: tonic::Request<()>) -> Result<tonic::Response<String>, tonic::Status> {
        Ok(tonic::Response::new("Pong".to_string()))
    }
    async fn metrics(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<String>, tonic::Status> {
        // Method implemented in HTTP server.
        Err(tonic::Status::not_found(ERR_NOT_FOUND))
    }
    async fn hook_traefik_self(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        // Method implemented in HTTP server.
        Err(tonic::Status::not_found(ERR_NOT_FOUND))
    }
    async fn hook_traefik_service(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        // Method implemented in HTTP server.
        Err(tonic::Status::not_found(ERR_NOT_FOUND))
    }
    async fn audit_list(
        &self,
        request: tonic::Request<pb::AuditListRequest>,
    ) -> Result<tonic::Response<pb::AuditListReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("audit_list", request)?;
        self.post(metrics, method::audit::list(self, request).await)
    }
    async fn audit_create(
        &self,
        request: tonic::Request<pb::AuditCreateRequest>,
    ) -> Result<tonic::Response<pb::AuditReadReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("audit_create", request)?;
        self.post(metrics, method::audit::create(self, request).await)
    }
    async fn audit_read(
        &self,
        request: tonic::Request<pb::AuditReadRequest>,
    ) -> Result<tonic::Response<pb::AuditReadReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("audit_read", request)?;
        self.post(metrics, method::audit::read(self, request).await)
    }
    async fn audit_update(
        &self,
        request: tonic::Request<pb::AuditUpdateRequest>,
    ) -> Result<tonic::Response<pb::AuditReadReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("audit_update", request)?;
        self.post(metrics, method::audit::update(self, request).await)
    }
    async fn key_list(
        &self,
        request: tonic::Request<pb::KeyListRequest>,
    ) -> Result<tonic::Response<pb::KeyListReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("key_list", request)?;
        self.post(metrics, method::key::list(self, request).await)
    }
    async fn key_create(
        &self,
        request: tonic::Request<pb::KeyCreateRequest>,
    ) -> Result<tonic::Response<pb::KeyCreateReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("key_create", request)?;
        self.post(metrics, method::key::create(self, request).await)
    }
    async fn key_read(
        &self,
        request: tonic::Request<pb::KeyReadRequest>,
    ) -> Result<tonic::Response<pb::KeyReadReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("key_read", request)?;
        self.post(metrics, method::key::read(self, request).await)
    }
    async fn key_update(
        &self,
        request: tonic::Request<pb::KeyUpdateRequest>,
    ) -> Result<tonic::Response<pb::KeyReadReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("key_update", request)?;
        self.post(metrics, method::key::update(self, request).await)
    }
    async fn key_delete(
        &self,
        request: tonic::Request<pb::KeyReadRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let (metrics, request) = self.pre_validate("key_delete", request)?;
        self.post(metrics, method::key::delete(self, request).await)
    }
    async fn service_list(
        &self,
        request: tonic::Request<pb::ServiceListRequest>,
    ) -> Result<tonic::Response<pb::ServiceListReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("service_list", request)?;
        self.post(metrics, method::service::list(self, request).await)
    }
    async fn service_create(
        &self,
        request: tonic::Request<pb::ServiceCreateRequest>,
    ) -> Result<tonic::Response<pb::ServiceReadReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("service_create", request)?;
        self.post(metrics, method::service::create(self, request).await)
    }
    async fn service_read(
        &self,
        request: tonic::Request<pb::ServiceReadRequest>,
    ) -> Result<tonic::Response<pb::ServiceReadReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("service_read", request)?;
        self.post(metrics, method::service::read(self, request).await)
    }
    async fn service_update(
        &self,
        request: tonic::Request<pb::ServiceUpdateRequest>,
    ) -> Result<tonic::Response<pb::ServiceReadReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("service_update", request)?;
        self.post(metrics, method::service::update(self, request).await)
    }
    async fn service_delete(
        &self,
        request: tonic::Request<pb::ServiceReadRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let (metrics, request) = self.pre_validate("service_delete", request)?;
        self.post(metrics, method::service::delete(self, request).await)
    }
    async fn user_list(
        &self,
        request: tonic::Request<pb::UserListRequest>,
    ) -> Result<tonic::Response<pb::UserListReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("user_list", request)?;
        self.post(metrics, method::user::list(self, request).await)
    }
    async fn user_create(
        &self,
        request: tonic::Request<pb::UserCreateRequest>,
    ) -> Result<tonic::Response<pb::UserCreateReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("user_create", request)?;
        self.post(metrics, method::user::create(self, request).await)
    }
    async fn user_read(
        &self,
        request: tonic::Request<pb::UserReadRequest>,
    ) -> Result<tonic::Response<pb::UserReadReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("user_read", request)?;
        self.post(metrics, method::user::read(self, request).await)
    }
    async fn user_update(
        &self,
        request: tonic::Request<pb::UserUpdateRequest>,
    ) -> Result<tonic::Response<pb::UserReadReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("user_update", request)?;
        self.post(metrics, method::user::update(self, request).await)
    }
    async fn user_delete(
        &self,
        request: tonic::Request<pb::UserReadRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let (metrics, request) = self.pre_validate("user_delete", request)?;
        self.post(metrics, method::user::delete(self, request).await)
    }
    async fn auth_key_verify(
        &self,
        request: tonic::Request<pb::AuthKeyRequest>,
    ) -> Result<tonic::Response<pb::AuthKeyReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_key_verify", request)?;
        self.post(metrics, method::auth::key::verify(self, request).await)
            .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_key_revoke(
        &self,
        request: tonic::Request<pb::AuthKeyRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_key_revoke", request)?;
        self.post(metrics, method::auth::key::revoke(self, request).await)
            .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_token_verify(
        &self,
        request: tonic::Request<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthTokenVerifyReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_token_verify", request)?;
        self.post(metrics, method::auth::token::verify(self, request).await)
            .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_token_refresh(
        &self,
        request: tonic::Request<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthTokenReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_token_refresh", request)?;
        self.post(metrics, method::auth::token::refresh(self, request).await)
            .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_token_revoke(
        &self,
        request: tonic::Request<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_token_revoke", request)?;
        self.post(metrics, method::auth::token::revoke(self, request).await)
            .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_totp_verify(
        &self,
        request: tonic::Request<pb::AuthTotpRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_totp_verify", request)?;
        self.post(metrics, method::auth::totp_verify(self, request).await)
            .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_csrf_create(
        &self,
        request: tonic::Request<pb::AuthCsrfCreateRequest>,
    ) -> Result<tonic::Response<pb::AuthCsrfCreateReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_csrf_create", request)?;
        self.post(metrics, method::auth::csrf_create(self, request).await)
            .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_csrf_verify(
        &self,
        request: tonic::Request<pb::AuthCsrfVerifyRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_csrf_verify", request)?;
        self.post(metrics, method::auth::csrf_verify(self, request).await)
            .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_local_login(
        &self,
        request: tonic::Request<pb::AuthLoginRequest>,
    ) -> Result<tonic::Response<pb::AuthLoginReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_local_login", request)?;
        self.post(metrics, method::auth::local::login(self, request).await)
            .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_local_register(
        &self,
        request: tonic::Request<pb::AuthRegisterRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_local_register", request)?;
        self.post(metrics, method::auth::local::register(self, request).await)
            .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_local_register_confirm(
        &self,
        request: tonic::Request<pb::AuthRegisterConfirmRequest>,
    ) -> Result<tonic::Response<pb::AuthPasswordMetaReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_local_register_confirm", request)?;
        self.post(
            metrics,
            method::auth::local::register_confirm(self, request).await,
        )
        .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_local_register_revoke(
        &self,
        request: tonic::Request<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_local_register_revoke", request)?;
        self.post(
            metrics,
            method::auth::local::register_revoke(self, request).await,
        )
        .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_local_reset_password(
        &self,
        request: tonic::Request<pb::AuthResetPasswordRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_local_reset_password", request)?;
        self.post(
            metrics,
            method::auth::local::reset_password(self, request).await,
        )
        .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_local_reset_password_confirm(
        &self,
        request: tonic::Request<pb::AuthResetPasswordConfirmRequest>,
    ) -> Result<tonic::Response<pb::AuthPasswordMetaReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_local_reset_password_confirm", request)?;
        self.post(
            metrics,
            method::auth::local::reset_password_confirm(self, request).await,
        )
        .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_local_reset_password_revoke(
        &self,
        request: tonic::Request<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_local_reset_password_revoke", request)?;
        self.post(
            metrics,
            method::auth::local::reset_password_revoke(self, request).await,
        )
        .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_local_update_email(
        &self,
        request: tonic::Request<pb::AuthUpdateEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_local_update_email", request)?;
        self.post(
            metrics,
            method::auth::local::update_email(self, request).await,
        )
        .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_local_update_email_revoke(
        &self,
        request: tonic::Request<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_local_update_email_revoke", request)?;
        self.post(
            metrics,
            method::auth::local::update_email_revoke(self, request).await,
        )
        .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_local_update_password(
        &self,
        request: tonic::Request<pb::AuthUpdatePasswordRequest>,
    ) -> Result<tonic::Response<pb::AuthPasswordMetaReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_local_update_password", request)?;
        self.post(
            metrics,
            method::auth::local::update_password(self, request).await,
        )
        .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_local_update_password_revoke(
        &self,
        request: tonic::Request<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_local_update_password_revoke", request)?;
        self.post(
            metrics,
            method::auth::local::update_password_revoke(self, request).await,
        )
        .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_github_oauth2_url(
        &self,
        request: tonic::Request<()>,
    ) -> Result<tonic::Response<pb::AuthOauth2UrlReply>, tonic::Status> {
        let (metrics, request) = self.pre("auth_github_oauth2_url", request)?;
        self.post(
            metrics,
            method::auth::github::oauth2_url(self, request).await,
        )
        .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_github_oauth2_callback(
        &self,
        request: tonic::Request<pb::AuthOauth2CallbackRequest>,
    ) -> Result<tonic::Response<pb::AuthTokenReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_github_oauth2_callback", request)?;
        self.post(
            metrics,
            method::auth::github::oauth2_callback(self, request).await,
        )
        .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_microsoft_oauth2_url(
        &self,
        request: tonic::Request<()>,
    ) -> Result<tonic::Response<pb::AuthOauth2UrlReply>, tonic::Status> {
        let (metrics, request) = self.pre("auth_microsoft_oauth2_url", request)?;
        self.post(
            metrics,
            method::auth::microsoft::oauth2_url(self, request).await,
        )
        .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
    async fn auth_microsoft_oauth2_callback(
        &self,
        request: tonic::Request<pb::AuthOauth2CallbackRequest>,
    ) -> Result<tonic::Response<pb::AuthTokenReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("auth_microsoft_oauth2_callback", request)?;
        self.post(
            metrics,
            method::auth::microsoft::oauth2_callback(self, request).await,
        )
        .map_err(|e| tonic::Status::new(e.code(), ERR_REDACTED))
    }
}
