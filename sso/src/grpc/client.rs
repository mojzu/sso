//! Blocking client.
use crate::{
    grpc::pb::{self, sso_client::SsoClient},
    User,
};
use http::{HeaderValue, Uri};
use std::convert::TryInto;
use std::fmt;
use std::str::FromStr;
use tokio::runtime::{Builder, Runtime};
use tonic::{transport::Channel, Status};

/// Split value of `Authorization` HTTP header into a type and value, where format is `VALUE` or `TYPE VALUE`.
/// For example `abc123def456`, `key abc123def456` and `token abc123def456`.
/// Without a type `key` is assumed and returned.
fn authorisation_type(type_value: String) -> Result<(String, String), Status> {
    let mut type_value = type_value.split_whitespace();
    let type_ = type_value.next();
    let type_: String = type_
        .ok_or_else(|| Status::unauthenticated("AuthTypeNotFound"))?
        .into();

    let value = type_value.next();
    if let Some(value) = value {
        Ok((type_, value.into()))
    } else {
        Ok(("key".to_owned(), type_))
    }
}

impl fmt::Debug for SsoClient<tonic::transport::Channel> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SsoClient {{ }}")
    }
}

impl SsoClient<tonic::transport::Channel> {
    pub async fn from_options(options: ClientOptions) -> Self {
        let authorisation = options.authorisation.to_owned();
        let user_authorisation = options.user_authorisation.to_owned();
        let channel = Channel::builder(options.uri.clone())
            .intercept_headers(move |headers| {
                headers.insert(
                    "Authorization",
                    HeaderValue::from_str(authorisation.as_ref()).unwrap(),
                );
                if let Some(user_authorisation) = &user_authorisation {
                    headers.insert(
                        "User-Authorization",
                        HeaderValue::from_str(user_authorisation).unwrap(),
                    );
                }
            })
            .connect()
            .await
            .unwrap();
        SsoClient::new(channel)
    }

    /// Authenticate user using token or key, returns user if successful.
    pub async fn authenticate(
        &mut self,
        key_or_token: Option<String>,
        audit: Option<String>,
    ) -> Result<(User, Option<String>), Status> {
        match key_or_token {
            Some(key_or_token) => {
                let (type_, value) = authorisation_type(key_or_token)?;
                match type_.as_ref() {
                    "key" => {
                        let res = self
                            .auth_key_verify(pb::AuthKeyRequest { key: value, audit })
                            .await?
                            .into_inner();
                        Ok((res.user.unwrap().try_into().unwrap(), res.audit))
                    }
                    "token" => {
                        let res = self
                            .auth_token_verify(pb::AuthTokenRequest {
                                token: value,
                                audit,
                            })
                            .await?
                            .into_inner();
                        Ok((res.user.unwrap().try_into().unwrap(), res.audit))
                    }
                    _ => Err(Status::unauthenticated("AuthTypeNotFound")),
                }
            }
            None => Err(Status::unauthenticated("AuthUndefined")),
        }
    }
}

pub type Client = SsoClient<tonic::transport::Channel>;

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

// The order of the fields in this struct is important. The runtime must be the first field and the
// client must be the last field so that when `ClientBlocking` is dropped the client is dropped
// before the runtime. Not doing this will result in a deadlock when dropped.
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
        let channel = rt.block_on(
            Channel::builder(options.uri.clone())
                .intercept_headers(move |headers| {
                    headers.insert(
                        "Authorization",
                        HeaderValue::from_str(authorisation.as_ref()).unwrap(),
                    );
                    if let Some(user_authorisation) = &user_authorisation {
                        headers.insert(
                            "User-Authorization",
                            HeaderValue::from_str(user_authorisation).unwrap(),
                        );
                    }
                })
                .connect(),
        )?;

        let client = SsoClient::new(channel);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_authorisation_type_none() {
        let (type_, value) = authorisation_type("abcdefg".to_owned()).unwrap();
        assert_eq!(type_, "key");
        assert_eq!(value, "abcdefg");
    }

    #[test]
    fn splits_authorisation_type_key() {
        let (type_, value) = authorisation_type("key abcdefg".to_owned()).unwrap();
        assert_eq!(type_, "key");
        assert_eq!(value, "abcdefg");
    }

    #[test]
    fn splits_authorisation_type_token() {
        let (type_, value) = authorisation_type("token abcdefg".to_owned()).unwrap();
        assert_eq!(type_, "token");
        assert_eq!(value, "abcdefg");
    }
}
