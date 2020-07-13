use crate::{grpc::pb::sso_client::SsoClient, prelude::*};
use http::Uri;
use std::{fmt, fs};
use tokio::runtime::{Builder, Runtime};
use tonic::{
    metadata::MetadataValue,
    transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity},
    Request, Status,
};

/// gRPC client channel TLS options.
#[derive(Debug, Clone)]
pub struct GrpcClientChannelTls {
    domain: Option<String>,
    cert: Option<Certificate>,
    identity: Option<Identity>,
}

impl Default for GrpcClientChannelTls {
    fn default() -> Self {
        Self {
            domain: None,
            cert: None,
            identity: None,
        }
    }
}

impl GrpcClientChannelTls {
    /// Construct gRPC client channel TLS options from named environment variables.
    pub fn from_env<T: AsRef<str>>(
        domain_name: T,
        ca_cert_name: T,
        client_cert_name: T,
        client_key_name: T,
    ) -> Self {
        let domain = env::string_opt(domain_name.as_ref());
        let cert = match env::string_opt(ca_cert_name.as_ref()) {
            Some(ca_cert) => {
                let ca_cert = fs::read(&ca_cert).expect("Failed to read TLS CA certificate file.");
                Some(Certificate::from_pem(ca_cert))
            }
            None => None,
        };
        let identity = if env::has_any_name(&[client_cert_name.as_ref(), client_key_name.as_ref()])
        {
            let client_cert = env::string(client_cert_name.as_ref())
                .expect("Failed to read TLS client certificate environment variable.");
            let client_cert =
                fs::read(&client_cert).expect("Failed to read TLS client certificate file.");
            let client_key = env::string(client_key_name.as_ref())
                .expect("Failed to read TLS client key environment variable.");
            let client_key = fs::read(&client_key).expect("Failed to read TLS client key file.");
            Some(Identity::from_pem(client_cert, client_key))
        } else {
            None
        };
        Self {
            domain,
            cert,
            identity,
        }
    }

    /// Return client TLS configuration if any TLS settings are defined.
    fn config(&self) -> Option<ClientTlsConfig> {
        let mut x = ClientTlsConfig::new();
        let mut tls_configured = false;
        if let Some(domain) = self.domain.as_ref() {
            x = x.domain_name(domain);
            tls_configured = true;
        }
        if let Some(cert) = self.cert.as_ref() {
            x = x.ca_certificate(cert.clone());
            tls_configured = true;
        }
        if let Some(identity) = self.identity.as_ref() {
            x = x.identity(identity.clone());
            tls_configured = true;
        }
        if tls_configured {
            Some(x)
        } else {
            None
        }
    }
}

/// gRPC client channel.
#[derive(Debug, Clone)]
pub struct GrpcClientChannel {
    inner: Channel,
}

impl GrpcClientChannel {
    /// Returns new gRPC client channel.
    pub async fn new<T: AsRef<str>>(uri: T, tls: GrpcClientChannelTls) -> DriverResult<Self> {
        let uri = Uri::from_str(uri.as_ref()).map_err(DriverError::HttpUri)?;
        let endpoint: Endpoint = uri.into();
        let endpoint = if let Some(tls_config) = tls.config() {
            endpoint.tls_config(tls_config).map_err(DriverError::TonicTransport)?
        } else {
            endpoint
        };
        let inner = endpoint
            .connect()
            .await
            .map_err(DriverError::TonicTransport)?;
        Ok(Self { inner })
    }

    /// Construct gRPC client channel from named environment variables.
    pub async fn from_env<T: AsRef<str>>(
        uri_name: T,
        domain_name: T,
        ca_cert_name: T,
        client_cert_name: T,
        client_key_name: T,
    ) -> DriverResult<Self> {
        let uri = env::string(uri_name.as_ref())
            .expect("Failed to read client URI environment variable.");
        let tls = GrpcClientChannelTls::from_env(
            domain_name,
            ca_cert_name,
            client_cert_name,
            client_key_name,
        );
        Self::new(uri, tls).await
    }

    /// Returns clone of tonic channel.
    pub fn channel(&self) -> Channel {
        self.inner.clone()
    }
}

/// gRPC client options.
#[derive(Debug, Clone)]
pub struct GrpcClientOptions {
    pub authorisation: Option<String>,
    pub user_authorisation: Option<String>,
    pub user_agent: Option<String>,
    pub forwarded: Option<String>,
}

impl Default for GrpcClientOptions {
    fn default() -> Self {
        Self {
            authorisation: None,
            user_authorisation: None,
            user_agent: None,
            forwarded: None,
        }
    }
}

impl GrpcClientOptions {
    pub fn authorisation(mut self, authorisation: Option<String>) -> Self {
        self.authorisation = authorisation;
        self
    }

    pub fn user_authorisation(mut self, user_authorisation: Option<String>) -> Self {
        self.user_authorisation = user_authorisation;
        self
    }

    pub fn user_agent(mut self, user_agent: Option<String>) -> Self {
        self.user_agent = user_agent;
        self
    }

    pub fn forwarded(mut self, forwarded: Option<String>) -> Self {
        self.forwarded = forwarded;
        self
    }

    fn interceptor(&self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let meta = req.metadata_mut();

        if let Some(authorisation) = self.authorisation.as_ref() {
            meta.insert(
                header::AUTHORISATION,
                MetadataValue::from_str(authorisation)
                    .map_err(|_e| Status::invalid_argument(ERR_INVALID_METADATA))?,
            );
        }
        if let Some(user_authorisation) = self.user_authorisation.as_ref() {
            meta.insert(
                header::USER_AUTHORISATION,
                MetadataValue::from_str(user_authorisation)
                    .map_err(|_e| Status::invalid_argument(ERR_INVALID_METADATA))?,
            );
        }
        if let Some(user_agent) = self.user_agent.as_ref() {
            meta.insert(
                header::USER_AGENT,
                MetadataValue::from_str(user_agent)
                    .map_err(|_e| Status::invalid_argument(ERR_INVALID_METADATA))?,
            );
        }
        if let Some(forwarded) = self.forwarded.as_ref() {
            meta.insert(
                header::X_FORWARDED_FOR,
                MetadataValue::from_str(forwarded)
                    .map_err(|_e| Status::invalid_argument(ERR_INVALID_METADATA))?,
            );
        }

        Ok(req)
    }
}

/// gRPC asynchronous client.
pub type GrpcClient = SsoClient<Channel>;

impl SsoClient<Channel> {
    /// Returns new gRPC asynchronous client.
    pub fn from_channel(channel: &GrpcClientChannel, options: GrpcClientOptions) -> Self {
        SsoClient::with_interceptor(channel.channel(), move |req: Request<()>| {
            options.interceptor(req)
        })
    }

    /// Authenticate user using headers, returns user if successful.
    /// If audit type argument is some, audit log ID is also returned.
    pub async fn authenticate(
        &mut self,
        auth: HeaderAuth,
        audit: Option<String>,
    ) -> Result<(User, Option<String>), Status> {
        match auth {
            HeaderAuth::Traefik(x) => match (x.user_key_id, x.user_id) {
                (Some(user_key_id), Some(user_id)) => {
                    let res = self
                        .user_read(pb::UserReadRequest::from_uuid(user_id))
                        .await?
                        .into_inner();
                    let user = res.data.expect("User is none.");

                    let audit = if let Some(audit) = audit {
                        let mut req = pb::AuditCreateRequest::default();
                        req.r#type = audit;
                        req.user_id = Some(pb::uuid_to_string(user_id));
                        req.user_key_id = Some(pb::uuid_to_string(user_key_id));
                        let res = self.audit_create(req).await?.into_inner();
                        Some(res.data.expect("Audit log is none.").id)
                    } else {
                        None
                    };

                    Ok((user.into(), audit))
                }
                _ => Err(Status::unauthenticated(ERR_AUTH_NOT_FOUND)),
            },
            HeaderAuth::Header(x) => match x {
                HeaderAuthType::Key(x) => {
                    let res = self
                        .auth_key_verify(pb::AuthKeyRequest { key: x, audit })
                        .await?
                        .into_inner();
                    Ok((res.user.expect("User is none.").into(), res.audit))
                }
                HeaderAuthType::Token(x) => {
                    let res = self
                        .auth_token_verify(pb::AuthTokenRequest { token: x, audit })
                        .await?
                        .into_inner();
                    Ok((res.user.expect("User is none.").into(), res.audit))
                }
            },
            HeaderAuth::None => Err(Status::unauthenticated(ERR_AUTH_NOT_FOUND)),
        }
    }
}

/// gRPC synchronous client.
pub struct GrpcClientBlocking {
    rt: Runtime,
    channel: GrpcClientChannel,
    client: SsoClient<Channel>,
}

impl fmt::Debug for GrpcClientBlocking {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GrpcClientBlocking {{ rt: {:?}, channel: {:?}, client }}",
            self.rt, self.channel
        )
    }
}

impl GrpcClientBlocking {
    /// Returns new gRPC synchronous client.
    pub fn new<T: AsRef<str>>(
        uri: T,
        tls: GrpcClientChannelTls,
        options: &GrpcClientOptions,
    ) -> DriverResult<Self> {
        let mut rt = Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .expect("Failed to build runtime.");

        let channel = rt.block_on(GrpcClientChannel::new(uri, tls))?;

        let options = options.clone();
        let client = SsoClient::with_interceptor(channel.channel(), move |req: Request<()>| {
            options.interceptor(req)
        });

        Ok(Self {
            rt,
            channel,
            client,
        })
    }

    /// Authenticate user using headers, returns user if successful.
    /// If audit type argument is some, audit log ID is also returned.
    pub async fn authenticate<T, TS>(
        &mut self,
        auth: HeaderAuth,
        audit: Option<String>,
    ) -> Result<(User, Option<String>), Status> {
        self.rt.block_on(self.client.authenticate(auth, audit))
    }

    pub fn ping(
        &mut self,
        request: impl tonic::IntoRequest<()>,
    ) -> Result<tonic::Response<String>, tonic::Status> {
        self.rt.block_on(self.client.ping(request))
    }

    pub fn metrics(
        &mut self,
        request: impl tonic::IntoRequest<()>,
    ) -> Result<tonic::Response<String>, tonic::Status> {
        self.rt.block_on(self.client.metrics(request))
    }

    pub fn audit_list(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuditListRequest>,
    ) -> Result<tonic::Response<pb::AuditListReply>, tonic::Status> {
        self.rt.block_on(self.client.audit_list(request))
    }

    pub fn audit_create(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuditCreateRequest>,
    ) -> Result<tonic::Response<pb::AuditReadReply>, tonic::Status> {
        self.rt.block_on(self.client.audit_create(request))
    }

    pub fn audit_read(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuditReadRequest>,
    ) -> Result<tonic::Response<pb::AuditReadReply>, tonic::Status> {
        self.rt.block_on(self.client.audit_read(request))
    }

    pub fn audit_update(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuditUpdateRequest>,
    ) -> Result<tonic::Response<pb::AuditReadReply>, tonic::Status> {
        self.rt.block_on(self.client.audit_update(request))
    }

    pub fn key_list(
        &mut self,
        request: impl tonic::IntoRequest<pb::KeyListRequest>,
    ) -> Result<tonic::Response<pb::KeyListReply>, tonic::Status> {
        self.rt.block_on(self.client.key_list(request))
    }

    pub fn key_create(
        &mut self,
        request: impl tonic::IntoRequest<pb::KeyCreateRequest>,
    ) -> Result<tonic::Response<pb::KeyCreateReply>, tonic::Status> {
        self.rt.block_on(self.client.key_create(request))
    }

    pub fn key_read(
        &mut self,
        request: impl tonic::IntoRequest<pb::KeyReadRequest>,
    ) -> Result<tonic::Response<pb::KeyReadReply>, tonic::Status> {
        self.rt.block_on(self.client.key_read(request))
    }

    pub fn key_update(
        &mut self,
        request: impl tonic::IntoRequest<pb::KeyUpdateRequest>,
    ) -> Result<tonic::Response<pb::KeyReadReply>, tonic::Status> {
        self.rt.block_on(self.client.key_update(request))
    }

    pub fn key_delete(
        &mut self,
        request: impl tonic::IntoRequest<pb::KeyReadRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        self.rt.block_on(self.client.key_delete(request))
    }

    pub fn service_list(
        &mut self,
        request: impl tonic::IntoRequest<pb::ServiceListRequest>,
    ) -> Result<tonic::Response<pb::ServiceListReply>, tonic::Status> {
        self.rt.block_on(self.client.service_list(request))
    }

    pub fn service_create(
        &mut self,
        request: impl tonic::IntoRequest<pb::ServiceCreateRequest>,
    ) -> Result<tonic::Response<pb::ServiceReadReply>, tonic::Status> {
        self.rt.block_on(self.client.service_create(request))
    }

    pub fn service_read(
        &mut self,
        request: impl tonic::IntoRequest<pb::ServiceReadRequest>,
    ) -> Result<tonic::Response<pb::ServiceReadReply>, tonic::Status> {
        self.rt.block_on(self.client.service_read(request))
    }

    pub fn service_update(
        &mut self,
        request: impl tonic::IntoRequest<pb::ServiceUpdateRequest>,
    ) -> Result<tonic::Response<pb::ServiceReadReply>, tonic::Status> {
        self.rt.block_on(self.client.service_update(request))
    }

    pub fn service_delete(
        &mut self,
        request: impl tonic::IntoRequest<pb::ServiceReadRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        self.rt.block_on(self.client.service_delete(request))
    }

    pub fn user_list(
        &mut self,
        request: impl tonic::IntoRequest<pb::UserListRequest>,
    ) -> Result<tonic::Response<pb::UserListReply>, tonic::Status> {
        self.rt.block_on(self.client.user_list(request))
    }

    pub fn user_create(
        &mut self,
        request: impl tonic::IntoRequest<pb::UserCreateRequest>,
    ) -> Result<tonic::Response<pb::UserCreateReply>, tonic::Status> {
        self.rt.block_on(self.client.user_create(request))
    }

    pub fn user_read(
        &mut self,
        request: impl tonic::IntoRequest<pb::UserReadRequest>,
    ) -> Result<tonic::Response<pb::UserReadReply>, tonic::Status> {
        self.rt.block_on(self.client.user_read(request))
    }

    pub fn user_update(
        &mut self,
        request: impl tonic::IntoRequest<pb::UserUpdateRequest>,
    ) -> Result<tonic::Response<pb::UserReadReply>, tonic::Status> {
        self.rt.block_on(self.client.user_update(request))
    }

    pub fn user_delete(
        &mut self,
        request: impl tonic::IntoRequest<pb::UserReadRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        self.rt.block_on(self.client.user_delete(request))
    }

    pub fn auth_key_verify(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthKeyRequest>,
    ) -> Result<tonic::Response<pb::AuthKeyReply>, tonic::Status> {
        self.rt.block_on(self.client.auth_key_verify(request))
    }

    pub fn auth_key_revoke(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthKeyRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        self.rt.block_on(self.client.auth_key_revoke(request))
    }

    pub fn auth_token_verify(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthTokenVerifyReply>, tonic::Status> {
        self.rt.block_on(self.client.auth_token_verify(request))
    }

    pub fn auth_token_refresh(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthTokenReply>, tonic::Status> {
        self.rt.block_on(self.client.auth_token_refresh(request))
    }

    pub fn auth_token_revoke(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        self.rt.block_on(self.client.auth_token_revoke(request))
    }

    pub fn auth_totp_verify(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthTotpRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        self.rt.block_on(self.client.auth_totp_verify(request))
    }

    pub fn auth_csrf_create(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthCsrfCreateRequest>,
    ) -> Result<tonic::Response<pb::AuthCsrfCreateReply>, tonic::Status> {
        self.rt.block_on(self.client.auth_csrf_create(request))
    }

    pub fn auth_csrf_verify(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthCsrfVerifyRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        self.rt.block_on(self.client.auth_csrf_verify(request))
    }

    pub fn auth_local_login(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthLoginRequest>,
    ) -> Result<tonic::Response<pb::AuthLoginReply>, tonic::Status> {
        self.rt.block_on(self.client.auth_local_login(request))
    }

    pub fn auth_local_register(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthRegisterRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        self.rt.block_on(self.client.auth_local_register(request))
    }

    pub fn auth_local_register_confirm(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthRegisterConfirmRequest>,
    ) -> Result<tonic::Response<pb::AuthPasswordMetaReply>, tonic::Status> {
        self.rt
            .block_on(self.client.auth_local_register_confirm(request))
    }

    pub fn auth_local_register_revoke(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        self.rt
            .block_on(self.client.auth_local_register_revoke(request))
    }

    pub fn auth_local_reset_password(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthResetPasswordRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        self.rt
            .block_on(self.client.auth_local_reset_password(request))
    }

    pub fn auth_local_reset_password_confirm(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthResetPasswordConfirmRequest>,
    ) -> Result<tonic::Response<pb::AuthPasswordMetaReply>, tonic::Status> {
        self.rt
            .block_on(self.client.auth_local_reset_password_confirm(request))
    }

    pub fn auth_local_reset_password_revoke(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        self.rt
            .block_on(self.client.auth_local_reset_password_revoke(request))
    }

    pub fn auth_local_update_email(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthUpdateEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        self.rt
            .block_on(self.client.auth_local_update_email(request))
    }

    pub fn auth_local_update_email_revoke(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        self.rt
            .block_on(self.client.auth_local_update_email_revoke(request))
    }

    pub fn auth_local_update_password(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthUpdatePasswordRequest>,
    ) -> Result<tonic::Response<pb::AuthPasswordMetaReply>, tonic::Status> {
        self.rt
            .block_on(self.client.auth_local_update_password(request))
    }

    pub fn auth_local_update_password_revoke(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthTokenRequest>,
    ) -> Result<tonic::Response<pb::AuthAuditReply>, tonic::Status> {
        self.rt
            .block_on(self.client.auth_local_update_password_revoke(request))
    }

    pub fn auth_github_oauth2_url(
        &mut self,
        request: impl tonic::IntoRequest<()>,
    ) -> Result<tonic::Response<pb::AuthOauth2UrlReply>, tonic::Status> {
        self.rt
            .block_on(self.client.auth_github_oauth2_url(request))
    }

    pub fn auth_github_oauth2_callback(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthOauth2CallbackRequest>,
    ) -> Result<tonic::Response<pb::AuthTokenReply>, tonic::Status> {
        self.rt
            .block_on(self.client.auth_github_oauth2_callback(request))
    }

    pub fn auth_microsoft_oauth2_url(
        &mut self,
        request: impl tonic::IntoRequest<()>,
    ) -> Result<tonic::Response<pb::AuthOauth2UrlReply>, tonic::Status> {
        self.rt
            .block_on(self.client.auth_microsoft_oauth2_url(request))
    }

    pub fn auth_microsoft_oauth2_callback(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuthOauth2CallbackRequest>,
    ) -> Result<tonic::Response<pb::AuthTokenReply>, tonic::Status> {
        self.rt
            .block_on(self.client.auth_microsoft_oauth2_callback(request))
    }
}
