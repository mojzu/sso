//! # Synchronous Client
use crate::{
    client::{Client, ClientActorOptions, ClientOptions, Error},
    core::User,
    server::api::{
        route, AuditCreateBody, AuditDataRequest, AuditListQuery, AuditListResponse,
        AuditReadResponse, AuthKeyBody, AuthKeyResponse, AuthLoginBody, AuthLoginResponse,
        AuthOauth2UrlResponse, AuthPasswordMetaResponse, AuthResetPasswordBody,
        AuthResetPasswordConfirmBody, AuthTokenBody, AuthTokenPartialResponse, AuthTokenResponse,
        AuthUpdateEmailBody, AuthUpdatePasswordBody, KeyCreateBody, KeyListQuery, KeyListResponse,
        KeyReadResponse, KeyUpdateBody, ServiceCreateBody, ServiceListQuery, ServiceListResponse,
        ServiceReadResponse, ServiceUpdateBody, UserCreateBody, UserCreateResponse, UserListQuery,
        UserListResponse, UserReadResponse, UserUpdateBody,
    },
};
use reqwest::{Client as ReqwestClient, Response};
use serde::ser::Serialize;
use serde_json::Value;
use uuid::Uuid;

/// Synchronous client.
#[derive(Clone)]
pub struct SyncClient {
    url: String,
    options: ClientOptions,
    client: ReqwestClient,
}

impl SyncClient {
    /// Create new client.
    pub fn new<T1: Into<String>>(
        url: T1,
        actor_options: ClientActorOptions,
        options: ClientOptions,
    ) -> Result<Self, Error> {
        let headers = actor_options.default_headers();
        let builder = reqwest::ClientBuilder::new()
            .use_rustls_tls()
            .default_headers(headers);
        let builder = match actor_options.crt_pem() {
            Some(buf) => {
                let crt_pem = reqwest::Certificate::from_pem(buf).unwrap();
                builder.add_root_certificate(crt_pem)
            }
            None => builder,
        };
        let builder = match actor_options.client_pem() {
            Some(buf) => {
                let client_pem = reqwest::Identity::from_pem(buf).unwrap();
                builder.identity(client_pem)
            }
            None => builder,
        };
        let client = builder.build().unwrap();

        Ok(Self {
            url: url.into(),
            options,
            client,
        })
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
            client: self.client.clone(),
        }
    }
}

impl SyncClient {
    /// Ping request.
    pub fn ping(&self) -> Result<Value, Error> {
        self.get(route::PING)
            .and_then(Client::response_json::<Value>)
    }

    /// Metrics request.
    pub fn metrics(&self) -> Result<String, Error> {
        self.get(route::METRICS).and_then(Client::response_text)
    }

    /// Authentication local provider login request.
    pub fn auth_local_login(&self, body: AuthLoginBody) -> Result<AuthLoginResponse, Error> {
        self.post_json(route::AUTH_LOCAL_LOGIN, &body)
            .and_then(Client::response_json::<AuthLoginResponse>)
    }

    /// Authentication local provider reset password request.
    pub fn auth_local_reset_password(&self, body: AuthResetPasswordBody) -> Result<(), Error> {
        self.post_json(route::AUTH_LOCAL_RESET_PASSWORD, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication local provider reset password confirm request.
    pub fn auth_local_reset_password_confirm(
        &self,
        body: AuthResetPasswordConfirmBody,
    ) -> Result<AuthPasswordMetaResponse, Error> {
        self.post_json(route::AUTH_LOCAL_RESET_PASSWORD_CONFIRM, &body)
            .and_then(Client::response_json::<AuthPasswordMetaResponse>)
    }

    /// Authentication local provider update email request.
    pub fn auth_local_update_email(&self, body: AuthUpdateEmailBody) -> Result<(), Error> {
        self.post_json(route::AUTH_LOCAL_UPDATE_EMAIL, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication local provider update email revoke request.
    pub fn auth_local_update_email_revoke(&self, body: AuthTokenBody) -> Result<(), Error> {
        self.post_json(route::AUTH_LOCAL_UPDATE_EMAIL_REVOKE, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication local provider update password request.
    pub fn auth_local_update_password(&self, body: AuthUpdatePasswordBody) -> Result<(), Error> {
        self.post_json(route::AUTH_LOCAL_UPDATE_PASSWORD, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication local provider update password revoke request.
    pub fn auth_local_update_password_revoke(&self, body: AuthTokenBody) -> Result<(), Error> {
        self.post_json(route::AUTH_LOCAL_UPDATE_PASSWORD_REVOKE, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication GitHub provider OAuth2 request.
    pub fn auth_github_oauth2_request(&self) -> Result<AuthOauth2UrlResponse, Error> {
        self.post(route::AUTH_GITHUB_OAUTH2)
            .and_then(Client::response_json::<AuthOauth2UrlResponse>)
    }

    /// Authentication Microsoft provider OAuth2 request.
    pub fn auth_microsoft_oauth2_request(&self) -> Result<AuthOauth2UrlResponse, Error> {
        self.post(route::AUTH_MICROSOFT_OAUTH2)
            .and_then(Client::response_json::<AuthOauth2UrlResponse>)
    }

    /// Authentication verify key.
    pub fn auth_key_verify(&self, body: AuthKeyBody) -> Result<AuthKeyResponse, Error> {
        self.post_json(route::AUTH_KEY_VERIFY, &body)
            .and_then(Client::response_json::<AuthKeyResponse>)
    }

    /// Authentication revoke key.
    pub fn auth_key_revoke(&self, body: AuthKeyBody) -> Result<(), Error> {
        self.post_json(route::AUTH_KEY_REVOKE, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication revoke token.
    pub fn auth_token_verify(
        &self,
        body: AuthTokenBody,
    ) -> Result<AuthTokenPartialResponse, Error> {
        self.post_json(route::AUTH_TOKEN_VERIFY, &body)
            .and_then(Client::response_json::<AuthTokenPartialResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_refresh(&self, body: AuthTokenBody) -> Result<AuthTokenResponse, Error> {
        self.post_json(route::AUTH_TOKEN_REFRESH, &body)
            .and_then(Client::response_json::<AuthTokenResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_revoke(&self, body: AuthTokenBody) -> Result<(), Error> {
        self.post_json(route::AUTH_TOKEN_REVOKE, &body)
            .and_then(Client::response_empty)
    }

    /// Audit list request.
    pub fn audit_list(&self, query: AuditListQuery) -> Result<AuditListResponse, Error> {
        self.get_query(route::AUDIT, &query)
            .and_then(Client::response_json::<AuditListResponse>)
    }

    /// Audit create request.
    pub fn audit_create(&self, body: AuditCreateBody) -> Result<AuditReadResponse, Error> {
        self.post_json(route::AUDIT, &body)
            .and_then(Client::response_json::<AuditReadResponse>)
    }

    /// Audit read by ID request.
    pub fn audit_read(&self, id: Uuid) -> Result<AuditReadResponse, Error> {
        let route = route::audit_id(id);
        self.get(&route)
            .and_then(Client::response_json::<AuditReadResponse>)
    }

    /// Key list request.
    pub fn key_list(&self, query: KeyListQuery) -> Result<KeyListResponse, Error> {
        self.get_query(route::KEY, &query)
            .and_then(Client::response_json::<KeyListResponse>)
    }

    /// Key create request.
    pub fn key_create(&self, body: KeyCreateBody) -> Result<KeyReadResponse, Error> {
        self.post_json(route::KEY, &body)
            .and_then(Client::response_json::<KeyReadResponse>)
    }

    /// Key read request.
    pub fn key_read(&self, id: Uuid) -> Result<KeyReadResponse, Error> {
        let route = route::key_id(id);
        self.get(&route)
            .and_then(Client::response_json::<KeyReadResponse>)
    }

    /// Key update request.
    pub fn key_update(&self, id: Uuid, body: KeyUpdateBody) -> Result<KeyReadResponse, Error> {
        let route = route::key_id(id);
        self.patch(&route, &body)
            .and_then(Client::response_json::<KeyReadResponse>)
    }

    /// Key delete request.
    pub fn key_delete(&self, id: Uuid) -> Result<(), Error> {
        let route = route::key_id(id);
        self.delete(&route).and_then(Client::response_empty)
    }

    /// Service list request.
    pub fn service_list(&self, query: ServiceListQuery) -> Result<ServiceListResponse, Error> {
        self.get_query(route::SERVICE, &query)
            .and_then(Client::response_json::<ServiceListResponse>)
    }

    /// Service create request.
    pub fn service_create(&self, body: ServiceCreateBody) -> Result<ServiceReadResponse, Error> {
        self.post_json(route::SERVICE, &body)
            .and_then(Client::response_json::<ServiceReadResponse>)
    }

    /// Service read request.
    pub fn service_read(&self, id: Uuid) -> Result<ServiceReadResponse, Error> {
        let route = route::service_id(id);
        self.get(&route)
            .and_then(Client::response_json::<ServiceReadResponse>)
    }

    /// Service update request.
    pub fn service_update(
        &self,
        id: Uuid,
        body: ServiceUpdateBody,
    ) -> Result<ServiceReadResponse, Error> {
        let route = route::service_id(id);
        self.patch(&route, &body)
            .and_then(Client::response_json::<ServiceReadResponse>)
    }

    /// Service delete request.
    pub fn service_delete(&self, id: Uuid) -> Result<(), Error> {
        let route = route::service_id(id);
        self.delete(&route).and_then(Client::response_empty)
    }

    /// User list request.
    pub fn user_list(&self, query: UserListQuery) -> Result<UserListResponse, Error> {
        self.get_query(route::USER, &query)
            .and_then(Client::response_json::<UserListResponse>)
    }

    /// User create request.
    pub fn user_create(&self, body: UserCreateBody) -> Result<UserCreateResponse, Error> {
        self.post_json(route::USER, &body)
            .and_then(Client::response_json::<UserCreateResponse>)
    }

    /// User read request.
    pub fn user_read(&self, id: Uuid) -> Result<UserReadResponse, Error> {
        let route = route::user_id(id);
        self.get(&route)
            .and_then(Client::response_json::<UserReadResponse>)
    }

    /// User update request.
    pub fn user_update(&self, id: Uuid, body: UserUpdateBody) -> Result<UserReadResponse, Error> {
        let route = route::user_id(id);
        self.patch(&route, &body)
            .and_then(Client::response_json::<UserReadResponse>)
    }

    /// User delete request.
    pub fn user_delete(&self, id: Uuid) -> Result<(), Error> {
        let route = route::user_id(id);
        self.delete(&route).and_then(Client::response_empty)
    }

    fn get(&self, route: &str) -> Result<Response, Error> {
        let url = Client::url(self.url(), route)?;
        self.options
            .request_headers(self.client.get(url))
            .send()
            .map_err(Into::into)
    }

    fn get_query<T: Serialize>(&self, route: &str, query: T) -> Result<Response, Error> {
        let url = Client::url_query(self.url(), route, query)?;
        self.options
            .request_headers(self.client.get(url))
            .send()
            .map_err(Into::into)
    }

    fn post(&self, route: &str) -> Result<Response, Error> {
        let url = Client::url(self.url(), route)?;
        self.options
            .request_headers(self.client.post(url))
            .send()
            .map_err(Into::into)
    }

    fn post_json<T: Serialize>(&self, route: &str, body: &T) -> Result<Response, Error> {
        let url = Client::url(self.url(), route)?;
        self.options
            .request_headers(self.client.post(url))
            .json(body)
            .send()
            .map_err(Into::into)
    }

    fn patch<T: Serialize>(&self, route: &str, body: &T) -> Result<Response, Error> {
        let url = Client::url(self.url(), route)?;
        self.options
            .request_headers(self.client.patch(url))
            .json(body)
            .send()
            .map_err(Into::into)
    }

    fn delete(&self, route: &str) -> Result<Response, Error> {
        let url = Client::url(self.url(), route)?;
        self.options
            .request_headers(self.client.delete(url))
            .send()
            .map_err(Into::into)
    }
}

impl SyncClient {
    /// Authenticate user using token or key, returns user if successful.
    pub fn authenticate(
        &self,
        key_or_token: Option<String>,
        audit: Option<AuditDataRequest>,
    ) -> Result<User, Error> {
        match key_or_token {
            Some(key_or_token) => {
                let (type_, value) = Client::authorisation_type(key_or_token)?;
                match type_.as_ref() {
                    "key" => {
                        let body = AuthKeyBody::new(value, audit);
                        self.auth_key_verify(body).map(|res| res.data.user_id)
                    }
                    "token" => {
                        let body = AuthTokenBody::new(value, audit);
                        self.auth_token_verify(body).map(|res| res.data.user_id)
                    }
                    _ => Err(Error::Forbidden),
                }
                .and_then(|user_id| self.user_read(user_id))
                .map(|res| res.data)
            }
            None => Err(Error::Forbidden),
        }
    }
}
