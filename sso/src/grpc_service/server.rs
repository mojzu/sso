use crate::{grpc_service::method, prelude::*};
use prometheus::{HistogramVec, IntCounterVec};
use std::fmt;

/// gRPC service server.
#[derive(Clone)]
pub struct GrpcServiceServer {
    authorisation: String,
    channel: GrpcClientChannel,
    traefik_enabled: bool,
    count: IntCounterVec,
    latency: HistogramVec,
}

impl fmt::Debug for GrpcServiceServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GrpcServiceServer {{ ... }}")
    }
}

impl GrpcServiceServer {
    /// Returns a new server.
    pub fn new<A: Into<String>>(
        authorisation: A,
        channel: GrpcClientChannel,
        traefik_enabled: bool,
        count: IntCounterVec,
        latency: HistogramVec,
    ) -> Self {
        Self {
            authorisation: authorisation.into(),
            channel,
            traefik_enabled,
            count,
            latency,
        }
    }

    /// Returns tonic service.
    pub fn service(self) -> pb::sso_service_server::SsoServiceServer<Self> {
        pb::sso_service_server::SsoServiceServer::new(self)
    }

    /// Returns new client using channel and headers from audit meta.
    pub fn client(&self, audit_meta: &AuditMeta) -> GrpcClient {
        let authorisation = Some(self.authorisation.to_owned());
        GrpcClient::from_channel(
            &self.channel,
            GrpcClientOptions::default()
                .authorisation(authorisation)
                .user_authorisation(audit_meta.user().map(|x| x.header_value()))
                .user_agent(Some(audit_meta.user_agent().to_owned()))
                .forwarded(audit_meta.forwarded().map(|x| x.to_owned())),
        )
    }

    fn pre(
        &self,
        path: &str,
        req: tonic::Request<()>,
    ) -> Result<(GrpcServerMetrics, GrpcMethodRequest<()>), tonic::Status> {
        let metrics = GrpcServerMetrics::start(path, &self.count, &self.latency);
        Ok((
            metrics,
            GrpcMethodRequest::from_unit(req, self.traefik_enabled)?,
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
            GrpcMethodRequest::from_request(req, self.traefik_enabled)?,
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
impl pb::sso_service_server::SsoService for GrpcServiceServer {
    async fn auth_local_login(
        &self,
        request: tonic::Request<pb::AuthLoginRequest>,
    ) -> Result<tonic::Response<pb::AuthLoginReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("local_login", request)?;
        self.post(metrics, method::local_login(self, request).await)
    }

    async fn auth_local_register(
        &self,
        request: tonic::Request<pb::AuthRegisterRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let (metrics, request) = self.pre_validate("local_register", request)?;
        self.post(metrics, method::local_register(self, request).await)
    }

    async fn auth_local_register_confirm(
        &self,
        request: tonic::Request<pb::AuthRegisterConfirmRequest>,
    ) -> Result<tonic::Response<pb::AuthPasswordMetaReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("local_register_confirm", request)?;
        self.post(metrics, method::local_register_confirm(self, request).await)
    }

    async fn auth_local_register_revoke(
        &self,
        request: tonic::Request<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("local_register_revoke", request)?;
        self.post(metrics, method::local_register_revoke(self, request).await)
    }

    async fn auth_local_reset_password(
        &self,
        request: tonic::Request<pb::AuthResetPasswordRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let (metrics, request) = self.pre_validate("local_reset_password", request)?;
        self.post(metrics, method::local_reset_password(self, request).await)
    }

    async fn auth_local_reset_password_confirm(
        &self,
        request: tonic::Request<pb::AuthResetPasswordConfirmRequest>,
    ) -> Result<tonic::Response<pb::AuthPasswordMetaReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("local_reset_password_confirm", request)?;
        self.post(
            metrics,
            method::local_reset_password_confirm(self, request).await,
        )
    }

    async fn auth_local_reset_password_revoke(
        &self,
        request: tonic::Request<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("local_reset_password_revoke", request)?;
        self.post(
            metrics,
            method::local_reset_password_revoke(self, request).await,
        )
    }

    async fn auth_local_update_email(
        &self,
        request: tonic::Request<pb::AuthUpdateEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let (metrics, request) = self.pre_validate("local_update_email", request)?;
        self.post(metrics, method::local_update_email(self, request).await)
    }

    async fn auth_local_update_email_revoke(
        &self,
        request: tonic::Request<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("local_update_email_revoke", request)?;
        self.post(
            metrics,
            method::local_update_email_revoke(self, request).await,
        )
    }

    async fn auth_local_update_password(
        &self,
        request: tonic::Request<pb::AuthUpdatePasswordRequest>,
    ) -> Result<tonic::Response<pb::AuthPasswordMetaReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("local_update_password", request)?;
        self.post(metrics, method::local_update_password(self, request).await)
    }

    async fn auth_local_update_password_revoke(
        &self,
        request: tonic::Request<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("local_update_password_revoke", request)?;
        self.post(
            metrics,
            method::local_update_password_revoke(self, request).await,
        )
    }

    async fn auth_microsoft_oauth2_url(
        &self,
        request: tonic::Request<()>,
    ) -> Result<tonic::Response<pb::AuthOauth2UrlReply>, tonic::Status> {
        let (metrics, request) = self.pre("microsoft_oauth2_url", request)?;
        self.post(metrics, method::microsoft_oauth2_url(self, request).await)
    }

    async fn auth_microsoft_oauth2_callback(
        &self,
        request: tonic::Request<pb::AuthOauth2CallbackRequest>,
    ) -> Result<tonic::Response<pb::AuthTokenReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("microsoft_oauth2_callback", request)?;
        self.post(
            metrics,
            method::microsoft_oauth2_callback(self, request).await,
        )
    }

    async fn auth_token_refresh(
        &self,
        request: tonic::Request<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthTokenReply>, tonic::Status> {
        let (metrics, request) = self.pre_validate("token_refresh", request)?;
        self.post(metrics, method::token_refresh(self, request).await)
    }
}
