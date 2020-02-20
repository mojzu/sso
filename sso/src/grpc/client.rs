use crate::{
    grpc::{
        pb::{self, sso_client::SsoClient},
        util::*,
    },
    HeaderAuth, HeaderAuthType, User,
};
use http::Uri;
use std::convert::TryInto;
use std::fmt;
use std::str::FromStr;
use tokio::runtime::{Builder, Runtime};
use tonic::{metadata::MetadataValue, transport::Endpoint, Request, Status};

impl fmt::Debug for SsoClient<tonic::transport::Channel> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SsoClient {{ }}")
    }
}

impl SsoClient<tonic::transport::Channel> {
    pub async fn from_options(options: ClientOptions) -> Self {
        let authorisation = options.authorisation.to_owned();
        let user_authorisation = options.user_authorisation.to_owned();

        let endpoint: Endpoint = options.uri.clone().into();
        let channel = endpoint
            .connect()
            .await
            .expect("Sso client connectioned failed.");

        SsoClient::with_interceptor(channel, move |mut req: Request<()>| {
            let meta = req.metadata_mut();

            meta.insert(
                "authorization",
                MetadataValue::from_str(authorisation.as_ref()).unwrap(),
            );
            if let Some(user_authorisation) = &user_authorisation {
                meta.insert(
                    "user-authorization",
                    MetadataValue::from_str(user_authorisation.as_ref()).unwrap(),
                );
            }

            Ok(req)
        })
    }

    /// Authenticate user using headers, returns user if successful.
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
                    let user = res.data.unwrap();

                    let audit = if let Some(audit) = audit {
                        let mut req = pb::AuditCreateRequest::default();
                        req.r#type = audit;
                        req.user_id = Some(uuid_to_string(user_id));
                        req.user_key_id = Some(uuid_to_string(user_key_id));
                        let res = self.audit_create(req).await?.into_inner();
                        Some(res.data.unwrap().id)
                    } else {
                        None
                    };

                    Ok((user.try_into().unwrap(), audit))
                }
                _ => Err(Status::unauthenticated(ERR_AUTH_NOT_FOUND)),
            },
            HeaderAuth::Header(x) => match x {
                HeaderAuthType::Key(x) => {
                    let res = self
                        .auth_key_verify(pb::AuthKeyRequest { key: x, audit })
                        .await?
                        .into_inner();
                    Ok((res.user.unwrap().try_into().unwrap(), res.audit))
                }
                HeaderAuthType::Token(x) => {
                    let res = self
                        .auth_token_verify(pb::AuthTokenRequest { token: x, audit })
                        .await?
                        .into_inner();
                    Ok((res.user.unwrap().try_into().unwrap(), res.audit))
                }
            },
            HeaderAuth::None => Err(Status::unauthenticated(ERR_AUTH_NOT_FOUND)),
        }
    }
}

/// gRPC asynchronous client.
pub type Client = SsoClient<tonic::transport::Channel>;

/// gRPC client options.
#[derive(Debug, Clone)]
pub struct ClientOptions {
    pub uri: Uri,
    pub authorisation: String,
    pub user_authorisation: Option<String>,
}

impl ClientOptions {
    pub fn new<U: AsRef<str>>(uri: U) -> Self {
        Self {
            uri: Uri::from_str(uri.as_ref()).unwrap(),
            authorisation: String::from(""),
            user_authorisation: None,
        }
    }

    pub fn authorisation<A: Into<String>>(mut self, authorisation: A) -> Self {
        self.authorisation = authorisation.into();
        self
    }

    pub fn user_authorisation(mut self, user_authorisation: Option<String>) -> Self {
        self.user_authorisation = user_authorisation;
        self
    }
}

/// gRPC synchronous client.
pub struct ClientBlocking {
    rt: Runtime,
    client: SsoClient<tonic::transport::Channel>,
}

impl fmt::Debug for ClientBlocking {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientBlocking {{ rt: {:?}, client }}", self.rt)
    }
}

impl ClientBlocking {
    pub fn connect(options: &ClientOptions) -> Result<Self, tonic::transport::Error> {
        let mut rt = Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap();

        let authorisation = options.authorisation.to_owned();
        let user_authorisation = options.user_authorisation.to_owned();

        let endpoint: Endpoint = options.uri.clone().into();
        let channel = rt.block_on(endpoint.connect())?;

        let client = SsoClient::with_interceptor(channel, move |mut req: Request<()>| {
            let meta = req.metadata_mut();

            meta.insert(
                "authorization",
                MetadataValue::from_str(authorisation.as_ref()).unwrap(),
            );
            if let Some(user_authorisation) = &user_authorisation {
                meta.insert(
                    "user-authorization",
                    MetadataValue::from_str(user_authorisation.as_ref()).unwrap(),
                );
            }

            Ok(req)
        });
        Ok(Self { rt, client })
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
