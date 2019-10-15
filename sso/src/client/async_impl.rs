use crate::{
    api_route,
    api_type::{
        AuditCreate2Request, AuditCreateRequest, AuditListRequest, AuditListResponse,
        AuditReadResponse, AuditUpdateRequest, AuthCsrfCreateRequest, AuthCsrfCreateResponse,
        AuthCsrfVerifyRequest, AuthKeyRequest, AuthKeyResponse, AuthLoginRequest,
        AuthLoginResponse, AuthOauth2CallbackRequest, AuthOauth2UrlResponse,
        AuthPasswordMetaResponse, AuthResetPasswordConfirmRequest, AuthResetPasswordRequest,
        AuthTokenAccessResponse, AuthTokenRequest, AuthTokenResponse, AuthTotpRequest,
        AuthUpdateEmailRequest, AuthUpdatePasswordRequest, KeyCreateRequest, KeyCreateResponse,
        KeyListRequest, KeyListResponse, KeyReadResponse, KeyUpdateRequest, ServiceCreateRequest,
        ServiceListRequest, ServiceListResponse, ServiceReadResponse, ServiceUpdateRequest,
        UserCreateRequest, UserCreateResponse, UserListRequest, UserListResponse, UserReadResponse,
        UserUpdateRequest,
    },
    client_msg::{Delete, Get, PatchJson, PostJson},
    Client, ClientActor, ClientActorRequest, ClientError, ClientOptions, User,
};
use actix::prelude::*;
use futures::future::Either;
use futures::{future, Future};
use serde_json::Value;
use uuid::Uuid;

/// Client (Asynchronous).
#[derive(Debug, Clone)]
pub struct ClientAsync {
    url: String,
    options: ClientOptions,
    addr: Addr<ClientActor>,
}

impl ClientAsync {
    /// Create new client handle.
    pub fn new<T1: Into<String>>(url: T1, options: ClientOptions, addr: Addr<ClientActor>) -> Self {
        ClientAsync {
            url: url.into(),
            options,
            addr,
        }
    }

    /// Returns url reference.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns options reference.
    pub fn options(&self) -> &ClientOptions {
        &self.options
    }

    /// Clone client with options.
    pub fn with_options(&self, options: ClientOptions) -> Self {
        Self {
            url: self.url.clone(),
            options,
            addr: self.addr.clone(),
        }
    }

    /// Clone client with forwarded (keeps options.authorisation).
    pub fn with_forwarded<T1: Into<String>>(&self, forwarded: T1) -> Self {
        let mut options = self.options.clone();
        options.forwarded = forwarded.into();
        self.with_options(options)
    }
}

impl ClientAsync {
    /// Ping request.
    pub fn ping(&self) -> impl Future<Item = Value, Error = ClientError> {
        self.addr
            .send(
                Get::new(self.url(), api_route::PING)
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<Value>)
    }

    /// Metrics request.
    pub fn metrics(&self) -> impl Future<Item = String, Error = ClientError> {
        self.addr
            .send(
                Get::new(self.url(), api_route::METRICS)
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(|res| res)
    }

    /// Authentication local provider login request.
    pub fn auth_local_login(
        &self,
        body: AuthLoginRequest,
    ) -> impl Future<Item = AuthLoginResponse, Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::AUTH_LOCAL_LOGIN, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthLoginResponse>)
    }

    /// Authentication local provider reset password request.
    pub fn auth_local_reset_password(
        &self,
        body: AuthResetPasswordRequest,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::AUTH_LOCAL_RESET_PASSWORD, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Authentication local provider reset password confirm request.
    pub fn auth_local_reset_password_confirm(
        &self,
        body: AuthResetPasswordConfirmRequest,
    ) -> impl Future<Item = AuthPasswordMetaResponse, Error = ClientError> {
        self.addr
            .send(
                PostJson::new(
                    self.url(),
                    api_route::AUTH_LOCAL_RESET_PASSWORD_CONFIRM,
                    Some(body),
                )
                .authorisation(self.options.authorisation())
                .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthPasswordMetaResponse>)
    }

    /// Authentication local provider update email request.
    pub fn auth_local_update_email(
        &self,
        body: AuthUpdateEmailRequest,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::AUTH_LOCAL_UPDATE_EMAIL, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Authentication local provider update email revoke request.
    pub fn auth_local_update_email_revoke(
        &self,
        body: AuthTokenRequest,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.addr
            .send(
                PostJson::new(
                    self.url(),
                    api_route::AUTH_LOCAL_UPDATE_EMAIL_REVOKE,
                    Some(body),
                )
                .authorisation(self.options.authorisation())
                .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Authentication local provider update password request.
    pub fn auth_local_update_password(
        &self,
        body: AuthUpdatePasswordRequest,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.addr
            .send(
                PostJson::new(
                    self.url(),
                    api_route::AUTH_LOCAL_UPDATE_PASSWORD,
                    Some(body),
                )
                .authorisation(self.options.authorisation())
                .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Authentication local provider update password revoke request.
    pub fn auth_local_update_password_revoke(
        &self,
        body: AuthTokenRequest,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.addr
            .send(
                PostJson::new(
                    self.url(),
                    api_route::AUTH_LOCAL_UPDATE_PASSWORD_REVOKE,
                    Some(body),
                )
                .authorisation(self.options.authorisation())
                .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Authentication GitHub provider OAuth2 url.
    pub fn auth_github_oauth2_url(
        &self,
    ) -> impl Future<Item = AuthOauth2UrlResponse, Error = ClientError> {
        self.addr
            .send(
                Get::new(self.url(), api_route::AUTH_GITHUB_OAUTH2)
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthOauth2UrlResponse>)
    }

    /// Authentication GitHub provider OAuth2 callback.
    pub fn auth_github_oauth2_callback(
        &self,
        body: AuthOauth2CallbackRequest,
    ) -> impl Future<Item = AuthTokenResponse, Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::AUTH_GITHUB_OAUTH2, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthTokenResponse>)
    }

    /// Authentication Microsoft provider OAuth2 url.
    pub fn auth_microsoft_oauth2_url(
        &self,
    ) -> impl Future<Item = AuthOauth2UrlResponse, Error = ClientError> {
        self.addr
            .send(
                Get::new(self.url(), api_route::AUTH_MICROSOFT_OAUTH2)
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthOauth2UrlResponse>)
    }

    /// Authentication Microsoft provider OAuth2 callback.
    pub fn auth_microsoft_oauth2_callback(
        &self,
        body: AuthOauth2CallbackRequest,
    ) -> impl Future<Item = AuthTokenResponse, Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::AUTH_MICROSOFT_OAUTH2, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthTokenResponse>)
    }

    /// Authentication verify key.
    pub fn auth_key_verify(
        &self,
        body: AuthKeyRequest,
    ) -> impl Future<Item = AuthKeyResponse, Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::AUTH_KEY_VERIFY, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthKeyResponse>)
    }

    /// Authentication revoke key.
    pub fn auth_key_revoke(
        &self,
        body: AuthKeyRequest,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::AUTH_KEY_REVOKE, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Authentication revoke token.
    pub fn auth_token_verify(
        &self,
        body: AuthTokenRequest,
    ) -> impl Future<Item = AuthTokenAccessResponse, Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::AUTH_TOKEN_VERIFY, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthTokenAccessResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_refresh(
        &self,
        body: AuthTokenRequest,
    ) -> impl Future<Item = AuthTokenResponse, Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::AUTH_TOKEN_REFRESH, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthTokenResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_revoke(
        &self,
        body: AuthTokenRequest,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::AUTH_TOKEN_REVOKE, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Authentication TOTP.
    pub fn auth_totp(&self, body: AuthTotpRequest) -> impl Future<Item = (), Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::AUTH_TOTP, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Authentication create CSRF.
    pub fn auth_csrf_create(
        &self,
        query: AuthCsrfCreateRequest,
    ) -> impl Future<Item = AuthCsrfCreateResponse, Error = ClientError> {
        let msg = Get::new(self.url(), api_route::AUTH_CSRF)
            .authorisation(self.options.authorisation())
            .forwarded(self.options.forwarded())
            .query(query)
            .unwrap();
        self.addr
            .send(msg)
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthCsrfCreateResponse>)
    }

    /// Authentication verify CSRF.
    pub fn auth_csrf_verify(
        &self,
        body: AuthCsrfVerifyRequest,
    ) -> impl Future<Item = (), Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::AUTH_CSRF, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Audit list request.
    pub fn audit_list(
        &self,
        query: AuditListRequest,
    ) -> impl Future<Item = AuditListResponse, Error = ClientError> {
        let msg = Get::new(self.url(), api_route::AUDIT)
            .authorisation(self.options.authorisation())
            .forwarded(self.options.forwarded())
            .query(query)
            .unwrap();
        self.addr
            .send(msg)
            .map_err(Into::into)
            .and_then(Client::result_json::<AuditListResponse>)
    }

    /// Audit create request.
    pub fn audit_create(
        &self,
        body: AuditCreateRequest,
    ) -> impl Future<Item = AuditReadResponse, Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::AUDIT, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuditReadResponse>)
    }

    /// Audit read request.
    pub fn audit_read(
        &self,
        id: Uuid,
    ) -> impl Future<Item = AuditReadResponse, Error = ClientError> {
        self.addr
            .send(
                Get::new(self.url(), api_route::audit_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuditReadResponse>)
    }

    /// Audit update request.
    pub fn audit_update(
        &self,
        id: Uuid,
        body: AuditUpdateRequest,
    ) -> impl Future<Item = AuditReadResponse, Error = ClientError> {
        self.addr
            .send(
                PatchJson::new(self.url(), api_route::audit_id(id), Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuditReadResponse>)
    }

    /// Key list request.
    pub fn key_list(
        &self,
        query: KeyListRequest,
    ) -> impl Future<Item = KeyListResponse, Error = ClientError> {
        let msg = Get::new(self.url(), api_route::KEY)
            .authorisation(self.options.authorisation())
            .forwarded(self.options.forwarded())
            .query(query)
            .unwrap();
        self.addr
            .send(msg)
            .map_err(Into::into)
            .and_then(Client::result_json::<KeyListResponse>)
    }

    /// Key create request.
    pub fn key_create(
        &self,
        body: KeyCreateRequest,
    ) -> impl Future<Item = KeyCreateResponse, Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::KEY, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<KeyCreateResponse>)
    }

    /// Key read request.
    pub fn key_read(&self, id: Uuid) -> impl Future<Item = KeyReadResponse, Error = ClientError> {
        self.addr
            .send(
                Get::new(self.url(), api_route::key_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<KeyReadResponse>)
    }

    /// Key update request.
    pub fn key_update(
        &self,
        id: Uuid,
        body: KeyUpdateRequest,
    ) -> impl Future<Item = KeyReadResponse, Error = ClientError> {
        self.addr
            .send(
                PatchJson::new(self.url(), api_route::key_id(id), Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<KeyReadResponse>)
    }

    /// Key delete request.
    pub fn key_delete(&self, id: Uuid) -> impl Future<Item = (), Error = ClientError> {
        self.addr
            .send(
                Delete::new(self.url(), api_route::key_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Service list request.
    pub fn service_list(
        &self,
        query: ServiceListRequest,
    ) -> impl Future<Item = ServiceListResponse, Error = ClientError> {
        let msg = Get::new(self.url(), api_route::SERVICE)
            .authorisation(self.options.authorisation())
            .forwarded(self.options.forwarded())
            .query(query)
            .unwrap();
        self.addr
            .send(msg)
            .map_err(Into::into)
            .and_then(Client::result_json::<ServiceListResponse>)
    }

    /// Service create request.
    pub fn service_create(
        &self,
        body: ServiceCreateRequest,
    ) -> impl Future<Item = ServiceReadResponse, Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::SERVICE, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<ServiceReadResponse>)
    }

    /// Service read request.
    pub fn service_read(
        &self,
        id: Uuid,
    ) -> impl Future<Item = ServiceReadResponse, Error = ClientError> {
        self.addr
            .send(
                Get::new(self.url(), api_route::service_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<ServiceReadResponse>)
    }

    /// Service update request.
    pub fn service_update(
        &self,
        id: Uuid,
        body: ServiceUpdateRequest,
    ) -> impl Future<Item = ServiceReadResponse, Error = ClientError> {
        self.addr
            .send(
                PatchJson::new(self.url(), api_route::service_id(id), Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<ServiceReadResponse>)
    }

    /// Service delete request.
    pub fn service_delete(&self, id: Uuid) -> impl Future<Item = (), Error = ClientError> {
        self.addr
            .send(
                Delete::new(self.url(), api_route::service_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// User list request.
    pub fn user_list(
        &self,
        query: UserListRequest,
    ) -> impl Future<Item = UserListResponse, Error = ClientError> {
        let msg = Get::new(self.url(), api_route::USER)
            .authorisation(self.options.authorisation())
            .forwarded(self.options.forwarded())
            .query(query)
            .unwrap();
        self.addr
            .send(msg)
            .map_err(Into::into)
            .and_then(Client::result_json::<UserListResponse>)
    }

    /// User create request.
    pub fn user_create(
        &self,
        body: UserCreateRequest,
    ) -> impl Future<Item = UserCreateResponse, Error = ClientError> {
        self.addr
            .send(
                PostJson::new(self.url(), api_route::USER, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<UserCreateResponse>)
    }

    /// User read request.
    pub fn user_read(&self, id: Uuid) -> impl Future<Item = UserReadResponse, Error = ClientError> {
        self.addr
            .send(
                Get::new(self.url(), api_route::user_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<UserReadResponse>)
    }

    /// User update request.
    pub fn user_update(
        &self,
        id: Uuid,
        body: UserUpdateRequest,
    ) -> impl Future<Item = UserReadResponse, Error = ClientError> {
        self.addr
            .send(
                PatchJson::new(self.url(), api_route::user_id(id), Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<UserReadResponse>)
    }

    /// User delete request.
    pub fn user_delete(&self, id: Uuid) -> impl Future<Item = (), Error = ClientError> {
        self.addr
            .send(
                Delete::new(self.url(), api_route::user_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }
}

impl ClientAsync {
    /// Authenticate user using token or key, returns user if successful.
    pub fn authenticate(
        &self,
        key_or_token: Option<String>,
        audit: Option<AuditCreate2Request>,
    ) -> impl Future<Item = (User, Option<Uuid>), Error = ClientError> {
        match key_or_token {
            Some(key_or_token) => match Client::authorisation_type(key_or_token) {
                Ok((type_, value)) => {
                    Either::A(Either::A(self.authenticate_inner(type_, value, audit)))
                }
                Err(e) => Either::A(Either::B(future::err(e))),
            },
            None => Either::B(future::err(ClientError::Unauthorised)),
        }
    }

    fn authenticate_inner(
        &self,
        type_: String,
        value: String,
        audit: Option<AuditCreate2Request>,
    ) -> impl Future<Item = (User, Option<Uuid>), Error = ClientError> {
        match type_.as_ref() {
            "key" => {
                let body = AuthKeyRequest::new(value, audit);
                Either::A(Either::A(
                    self.auth_key_verify(body)
                        .map(|res| (res.data.user, res.audit)),
                ))
            }
            "token" => {
                let body = AuthTokenRequest::new(value, audit);
                Either::A(Either::B(
                    self.auth_token_verify(body)
                        .map(|res| (res.data.user, res.audit)),
                ))
            }
            _ => Either::B(future::err(ClientError::Unauthorised)),
        }
    }
}
