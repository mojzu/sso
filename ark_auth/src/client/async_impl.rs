//! # Asynchronous Client
use crate::client::{
    Client, ClientExecutor, ClientExecutorRequest, ClientOptions, Delete, Error, Get, PatchJson,
    PostJson,
};
use crate::core::User;
use crate::server::api::{
    route, AuditCreateBody, AuditDataRequest, AuditListQuery, AuditListResponse, AuditReadResponse,
    AuthKeyBody, AuthKeyResponse, AuthLoginBody, AuthLoginResponse, AuthOauth2UrlResponse,
    AuthPasswordMetaResponse, AuthResetPasswordBody, AuthResetPasswordConfirmBody, AuthTokenBody,
    AuthTokenPartialResponse, AuthTokenResponse, AuthUpdateEmailBody, AuthUpdatePasswordBody,
    KeyCreateBody, KeyListQuery, KeyListResponse, KeyReadResponse, KeyUpdateBody,
    ServiceCreateBody, ServiceListQuery, ServiceListResponse, ServiceReadResponse,
    ServiceUpdateBody, UserCreateBody, UserCreateResponse, UserListQuery, UserListResponse,
    UserReadResponse, UserUpdateBody,
};
use actix::prelude::*;
use futures::future::Either;
use futures::{future, Future};
use serde_json::Value;

/// Asynchronous client handle.
#[derive(Clone)]
pub struct AsyncClient {
    url: String,
    options: ClientOptions,
    addr: Addr<ClientExecutor>,
}

impl AsyncClient {
    /// Create new client handle.
    pub fn new<T1: Into<String>>(
        url: T1,
        options: ClientOptions,
        addr: Addr<ClientExecutor>,
    ) -> Self {
        AsyncClient {
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

impl AsyncClient {
    /// Ping request.
    pub fn ping(&self) -> impl Future<Item = Value, Error = Error> {
        self.addr
            .send(
                Get::new(self.url(), route::PING)
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<Value>)
    }

    /// Metrics request.
    pub fn metrics(&self) -> impl Future<Item = String, Error = Error> {
        self.addr
            .send(
                Get::new(self.url(), route::METRICS)
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(|res| res)
    }

    /// Authentication local provider login request.
    pub fn auth_local_login(
        &self,
        body: AuthLoginBody,
    ) -> impl Future<Item = AuthLoginResponse, Error = Error> {
        self.addr
            .send(
                PostJson::new(self.url(), route::AUTH_LOCAL_LOGIN, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthLoginResponse>)
    }

    /// Authentication local provider reset password request.
    pub fn auth_local_reset_password(
        &self,
        body: AuthResetPasswordBody,
    ) -> impl Future<Item = (), Error = Error> {
        self.addr
            .send(
                PostJson::new(self.url(), route::AUTH_LOCAL_RESET_PASSWORD, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Authentication local provider reset password confirm request.
    pub fn auth_local_reset_password_confirm(
        &self,
        body: AuthResetPasswordConfirmBody,
    ) -> impl Future<Item = AuthPasswordMetaResponse, Error = Error> {
        self.addr
            .send(
                PostJson::new(
                    self.url(),
                    route::AUTH_LOCAL_RESET_PASSWORD_CONFIRM,
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
        body: AuthUpdateEmailBody,
    ) -> impl Future<Item = (), Error = Error> {
        self.addr
            .send(
                PostJson::new(self.url(), route::AUTH_LOCAL_UPDATE_EMAIL, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Authentication local provider update email revoke request.
    pub fn auth_local_update_email_revoke(
        &self,
        body: AuthTokenBody,
    ) -> impl Future<Item = (), Error = Error> {
        self.addr
            .send(
                PostJson::new(
                    self.url(),
                    route::AUTH_LOCAL_UPDATE_EMAIL_REVOKE,
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
        body: AuthUpdatePasswordBody,
    ) -> impl Future<Item = (), Error = Error> {
        self.addr
            .send(
                PostJson::new(self.url(), route::AUTH_LOCAL_UPDATE_PASSWORD, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Authentication local provider update password revoke request.
    pub fn auth_local_update_password_revoke(
        &self,
        body: AuthTokenBody,
    ) -> impl Future<Item = (), Error = Error> {
        self.addr
            .send(
                PostJson::new(
                    self.url(),
                    route::AUTH_LOCAL_UPDATE_PASSWORD_REVOKE,
                    Some(body),
                )
                .authorisation(self.options.authorisation())
                .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Authentication GitHub provider OAuth2 request.
    pub fn auth_github_oauth2_request(
        &self,
    ) -> impl Future<Item = AuthOauth2UrlResponse, Error = Error> {
        self.addr
            .send(
                PostJson::<Value>::new(self.url(), route::AUTH_GITHUB_OAUTH2, None)
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthOauth2UrlResponse>)
    }

    /// Authentication Microsoft provider OAuth2 request.
    pub fn auth_microsoft_oauth2_request(
        &self,
    ) -> impl Future<Item = AuthOauth2UrlResponse, Error = Error> {
        self.addr
            .send(
                PostJson::<Value>::new(self.url(), route::AUTH_MICROSOFT_OAUTH2, None)
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthOauth2UrlResponse>)
    }

    /// Authentication verify key.
    pub fn auth_key_verify(
        &self,
        body: AuthKeyBody,
    ) -> impl Future<Item = AuthKeyResponse, Error = Error> {
        self.addr
            .send(
                PostJson::new(self.url(), route::AUTH_KEY_VERIFY, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthKeyResponse>)
    }

    /// Authentication revoke key.
    pub fn auth_key_revoke(&self, body: AuthKeyBody) -> impl Future<Item = (), Error = Error> {
        self.addr
            .send(
                PostJson::new(self.url(), route::AUTH_KEY_REVOKE, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Authentication revoke token.
    pub fn auth_token_verify(
        &self,
        body: AuthTokenBody,
    ) -> impl Future<Item = AuthTokenPartialResponse, Error = Error> {
        self.addr
            .send(
                PostJson::new(self.url(), route::AUTH_TOKEN_VERIFY, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthTokenPartialResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_refresh(
        &self,
        body: AuthTokenBody,
    ) -> impl Future<Item = AuthTokenResponse, Error = Error> {
        self.addr
            .send(
                PostJson::new(self.url(), route::AUTH_TOKEN_REFRESH, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuthTokenResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_revoke(&self, body: AuthTokenBody) -> impl Future<Item = (), Error = Error> {
        self.addr
            .send(
                PostJson::new(self.url(), route::AUTH_TOKEN_REVOKE, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Audit list request.
    pub fn audit_list(
        &self,
        query: AuditListQuery,
    ) -> impl Future<Item = AuditListResponse, Error = Error> {
        let msg = Get::new(self.url(), route::AUDIT)
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
        body: AuditCreateBody,
    ) -> impl Future<Item = AuditReadResponse, Error = Error> {
        self.addr
            .send(
                PostJson::new(self.url(), route::AUDIT, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuditReadResponse>)
    }

    /// Audit read by ID request.
    pub fn audit_read(&self, id: &str) -> impl Future<Item = AuditReadResponse, Error = Error> {
        self.addr
            .send(
                Get::new(self.url(), route::audit_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<AuditReadResponse>)
    }

    /// Key list request.
    pub fn key_list(
        &self,
        query: KeyListQuery,
    ) -> impl Future<Item = KeyListResponse, Error = Error> {
        let msg = Get::new(self.url(), route::KEY)
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
        body: KeyCreateBody,
    ) -> impl Future<Item = KeyReadResponse, Error = Error> {
        self.addr
            .send(
                PostJson::new(self.url(), route::KEY, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<KeyReadResponse>)
    }

    /// Key read request.
    pub fn key_read(&self, id: &str) -> impl Future<Item = KeyReadResponse, Error = Error> {
        self.addr
            .send(
                Get::new(self.url(), route::key_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<KeyReadResponse>)
    }

    /// Key update request.
    pub fn key_update(
        &self,
        id: &str,
        body: KeyUpdateBody,
    ) -> impl Future<Item = KeyReadResponse, Error = Error> {
        self.addr
            .send(
                PatchJson::new(self.url(), route::key_id(id), Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<KeyReadResponse>)
    }

    /// Key delete request.
    pub fn key_delete(&self, id: &str) -> impl Future<Item = (), Error = Error> {
        self.addr
            .send(
                Delete::new(self.url(), route::key_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// Service list request.
    pub fn service_list(
        &self,
        query: ServiceListQuery,
    ) -> impl Future<Item = ServiceListResponse, Error = Error> {
        let msg = Get::new(self.url(), route::SERVICE)
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
        body: ServiceCreateBody,
    ) -> impl Future<Item = ServiceReadResponse, Error = Error> {
        self.addr
            .send(
                PostJson::new(self.url(), route::SERVICE, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<ServiceReadResponse>)
    }

    /// Service read request.
    pub fn service_read(&self, id: &str) -> impl Future<Item = ServiceReadResponse, Error = Error> {
        self.addr
            .send(
                Get::new(self.url(), route::service_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<ServiceReadResponse>)
    }

    /// Service update request.
    pub fn service_update(
        &self,
        id: &str,
        body: ServiceUpdateBody,
    ) -> impl Future<Item = ServiceReadResponse, Error = Error> {
        self.addr
            .send(
                PatchJson::new(self.url(), route::service_id(id), Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<ServiceReadResponse>)
    }

    /// Service delete request.
    pub fn service_delete(&self, id: &str) -> impl Future<Item = (), Error = Error> {
        self.addr
            .send(
                Delete::new(self.url(), route::service_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }

    /// User list request.
    pub fn user_list(
        &self,
        query: UserListQuery,
    ) -> impl Future<Item = UserListResponse, Error = Error> {
        let msg = Get::new(self.url(), route::USER)
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
        body: UserCreateBody,
    ) -> impl Future<Item = UserCreateResponse, Error = Error> {
        self.addr
            .send(
                PostJson::new(self.url(), route::USER, Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<UserCreateResponse>)
    }

    /// User read request.
    pub fn user_read(&self, id: &str) -> impl Future<Item = UserReadResponse, Error = Error> {
        self.addr
            .send(
                Get::new(self.url(), route::user_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<UserReadResponse>)
    }

    /// User update request.
    pub fn user_update(
        &self,
        id: &str,
        body: UserUpdateBody,
    ) -> impl Future<Item = UserReadResponse, Error = Error> {
        self.addr
            .send(
                PatchJson::new(self.url(), route::user_id(id), Some(body))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_json::<UserReadResponse>)
    }

    /// User delete request.
    pub fn user_delete(&self, id: &str) -> impl Future<Item = (), Error = Error> {
        self.addr
            .send(
                Delete::new(self.url(), route::user_id(id))
                    .authorisation(self.options.authorisation())
                    .forwarded(self.options.forwarded()),
            )
            .map_err(Into::into)
            .and_then(Client::result_empty)
    }
}

impl AsyncClient {
    /// Authenticate user using token or key, returns user if successful.
    pub fn authenticate(
        &self,
        key_or_token: Option<String>,
        audit: Option<AuditDataRequest>,
    ) -> impl Future<Item = User, Error = Error> {
        match key_or_token {
            Some(key_or_token) => match Client::authorisation_type(key_or_token) {
                Ok((type_, value)) => {
                    let s1 = self.clone();
                    Either::A(Either::A(
                        self.authenticate_inner(type_, value, audit)
                            .and_then(move |user_id| s1.user_read(&user_id))
                            .map(|res| res.data),
                    ))
                }
                Err(e) => Either::A(Either::B(future::err(e))),
            },
            None => Either::B(future::err(Error::Forbidden)),
        }
    }

    fn authenticate_inner(
        &self,
        type_: String,
        value: String,
        audit: Option<AuditDataRequest>,
    ) -> impl Future<Item = String, Error = Error> {
        match type_.as_ref() {
            "key" => {
                let body = AuthKeyBody::new(value, audit);
                Either::A(Either::A(
                    self.auth_key_verify(body).map(|res| res.data.user_id),
                ))
            }
            "token" => {
                let body = AuthTokenBody::new(value, audit);
                Either::A(Either::B(
                    self.auth_token_verify(body).map(|res| res.data.user_id),
                ))
            }
            _ => Either::B(future::err(Error::Forbidden)),
        }
    }
}
