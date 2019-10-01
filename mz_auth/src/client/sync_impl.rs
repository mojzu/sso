use crate::{
    api_types::{
        AuditCreateRequest, AuditDataRequest, AuditListRequest, AuditListResponse,
        AuditReadResponse, AuthOauth2UrlResponse,
    },
    server_api::{
        route, AuthKeyBody, AuthKeyResponse, AuthLoginBody, AuthLoginResponse,
        AuthPasswordMetaResponse, AuthResetPasswordBody, AuthResetPasswordConfirmBody,
        AuthTokenAccessResponse, AuthTokenBody, AuthTokenResponse, AuthTotpBody,
        AuthUpdateEmailBody, AuthUpdatePasswordBody, KeyCreateBody, KeyListQuery, KeyListResponse,
        KeyReadResponse, KeyUpdateBody, ServiceCreateBody, ServiceListQuery, ServiceListResponse,
        ServiceReadResponse, ServiceUpdateBody, UserCreateBody, UserCreateResponse, UserListQuery,
        UserListResponse, UserReadResponse, UserUpdateBody,
    },
    Client, ClientActorOptions, ClientError, ClientOptions, ClientResult, User,
};
use reqwest::{Client as ReqwestClient, Response};
use serde::ser::Serialize;
use serde_json::Value;
use uuid::Uuid;

/// Client (Synchronous).
#[derive(Clone)]
pub struct ClientSync {
    url: String,
    options: ClientOptions,
    client: ReqwestClient,
}

impl ClientSync {
    /// Create new client.
    pub fn new<T1: Into<String>>(
        url: T1,
        actor_options: ClientActorOptions,
        options: ClientOptions,
    ) -> ClientResult<Self> {
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

    /// Returns client reference.
    pub fn client(&self) -> &ReqwestClient {
        &self.client
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

impl ClientSync {
    /// Ping request.
    pub fn ping(&self) -> ClientResult<Value> {
        self.get(route::PING)
            .and_then(Client::response_json::<Value>)
    }

    /// Metrics request.
    pub fn metrics(&self) -> ClientResult<String> {
        self.get(route::METRICS).and_then(Client::response_text)
    }

    /// Authentication local provider login request.
    pub fn auth_local_login(&self, body: AuthLoginBody) -> ClientResult<AuthLoginResponse> {
        self.post_json(route::AUTH_LOCAL_LOGIN, &body)
            .and_then(Client::response_json::<AuthLoginResponse>)
    }

    /// Authentication local provider reset password request.
    pub fn auth_local_reset_password(&self, body: AuthResetPasswordBody) -> ClientResult<()> {
        self.post_json(route::AUTH_LOCAL_RESET_PASSWORD, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication local provider reset password confirm request.
    pub fn auth_local_reset_password_confirm(
        &self,
        body: AuthResetPasswordConfirmBody,
    ) -> ClientResult<AuthPasswordMetaResponse> {
        self.post_json(route::AUTH_LOCAL_RESET_PASSWORD_CONFIRM, &body)
            .and_then(Client::response_json::<AuthPasswordMetaResponse>)
    }

    /// Authentication local provider update email request.
    pub fn auth_local_update_email(&self, body: AuthUpdateEmailBody) -> ClientResult<()> {
        self.post_json(route::AUTH_LOCAL_UPDATE_EMAIL, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication local provider update email revoke request.
    pub fn auth_local_update_email_revoke(&self, body: AuthTokenBody) -> ClientResult<()> {
        self.post_json(route::AUTH_LOCAL_UPDATE_EMAIL_REVOKE, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication local provider update password request.
    pub fn auth_local_update_password(&self, body: AuthUpdatePasswordBody) -> ClientResult<()> {
        self.post_json(route::AUTH_LOCAL_UPDATE_PASSWORD, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication local provider update password revoke request.
    pub fn auth_local_update_password_revoke(&self, body: AuthTokenBody) -> ClientResult<()> {
        self.post_json(route::AUTH_LOCAL_UPDATE_PASSWORD_REVOKE, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication GitHub provider OAuth2 request.
    pub fn auth_github_oauth2_request(&self) -> ClientResult<AuthOauth2UrlResponse> {
        self.post(route::AUTH_GITHUB_OAUTH2)
            .and_then(Client::response_json::<AuthOauth2UrlResponse>)
    }

    /// Authentication Microsoft provider OAuth2 request.
    pub fn auth_microsoft_oauth2_request(&self) -> ClientResult<AuthOauth2UrlResponse> {
        self.post(route::AUTH_MICROSOFT_OAUTH2)
            .and_then(Client::response_json::<AuthOauth2UrlResponse>)
    }

    /// Authentication verify key.
    pub fn auth_key_verify(&self, body: AuthKeyBody) -> ClientResult<AuthKeyResponse> {
        self.post_json(route::AUTH_KEY_VERIFY, &body)
            .and_then(Client::response_json::<AuthKeyResponse>)
    }

    /// Authentication revoke key.
    pub fn auth_key_revoke(&self, body: AuthKeyBody) -> ClientResult<()> {
        self.post_json(route::AUTH_KEY_REVOKE, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication revoke token.
    pub fn auth_token_verify(&self, body: AuthTokenBody) -> ClientResult<AuthTokenAccessResponse> {
        self.post_json(route::AUTH_TOKEN_VERIFY, &body)
            .and_then(Client::response_json::<AuthTokenAccessResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_refresh(&self, body: AuthTokenBody) -> ClientResult<AuthTokenResponse> {
        self.post_json(route::AUTH_TOKEN_REFRESH, &body)
            .and_then(Client::response_json::<AuthTokenResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_revoke(&self, body: AuthTokenBody) -> ClientResult<()> {
        self.post_json(route::AUTH_TOKEN_REVOKE, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication TOTP.
    pub fn auth_totp(&self, body: AuthTotpBody) -> ClientResult<()> {
        self.post_json(route::AUTH_TOTP, &body)
            .and_then(Client::response_empty)
    }

    /// Audit list request.
    pub fn audit_list(&self, query: AuditListRequest) -> ClientResult<AuditListResponse> {
        self.get_query(route::AUDIT, &query)
            .and_then(Client::response_json::<AuditListResponse>)
    }

    /// Audit create request.
    pub fn audit_create(&self, body: AuditCreateRequest) -> ClientResult<AuditReadResponse> {
        self.post_json(route::AUDIT, &body)
            .and_then(Client::response_json::<AuditReadResponse>)
    }

    /// Audit read by ID request.
    pub fn audit_read(&self, id: Uuid) -> ClientResult<AuditReadResponse> {
        let route = route::audit_id(id);
        self.get(&route)
            .and_then(Client::response_json::<AuditReadResponse>)
    }

    /// Key list request.
    pub fn key_list(&self, query: KeyListQuery) -> ClientResult<KeyListResponse> {
        self.get_query(route::KEY, &query)
            .and_then(Client::response_json::<KeyListResponse>)
    }

    /// Key create request.
    pub fn key_create(&self, body: KeyCreateBody) -> ClientResult<KeyReadResponse> {
        self.post_json(route::KEY, &body)
            .and_then(Client::response_json::<KeyReadResponse>)
    }

    /// Key read request.
    pub fn key_read(&self, id: Uuid) -> ClientResult<KeyReadResponse> {
        let route = route::key_id(id);
        self.get(&route)
            .and_then(Client::response_json::<KeyReadResponse>)
    }

    /// Key update request.
    pub fn key_update(&self, id: Uuid, body: KeyUpdateBody) -> ClientResult<KeyReadResponse> {
        let route = route::key_id(id);
        self.patch(&route, &body)
            .and_then(Client::response_json::<KeyReadResponse>)
    }

    /// Key delete request.
    pub fn key_delete(&self, id: Uuid) -> ClientResult<()> {
        let route = route::key_id(id);
        self.delete(&route).and_then(Client::response_empty)
    }

    /// Service list request.
    pub fn service_list(&self, query: ServiceListQuery) -> ClientResult<ServiceListResponse> {
        self.get_query(route::SERVICE, &query)
            .and_then(Client::response_json::<ServiceListResponse>)
    }

    /// Service create request.
    pub fn service_create(&self, body: ServiceCreateBody) -> ClientResult<ServiceReadResponse> {
        self.post_json(route::SERVICE, &body)
            .and_then(Client::response_json::<ServiceReadResponse>)
    }

    /// Service read request.
    pub fn service_read(&self, id: Uuid) -> ClientResult<ServiceReadResponse> {
        let route = route::service_id(id);
        self.get(&route)
            .and_then(Client::response_json::<ServiceReadResponse>)
    }

    /// Service update request.
    pub fn service_update(
        &self,
        id: Uuid,
        body: ServiceUpdateBody,
    ) -> ClientResult<ServiceReadResponse> {
        let route = route::service_id(id);
        self.patch(&route, &body)
            .and_then(Client::response_json::<ServiceReadResponse>)
    }

    /// Service delete request.
    pub fn service_delete(&self, id: Uuid) -> ClientResult<()> {
        let route = route::service_id(id);
        self.delete(&route).and_then(Client::response_empty)
    }

    /// User list request.
    pub fn user_list(&self, query: UserListQuery) -> ClientResult<UserListResponse> {
        self.get_query(route::USER, &query)
            .and_then(Client::response_json::<UserListResponse>)
    }

    /// User create request.
    pub fn user_create(&self, body: UserCreateBody) -> ClientResult<UserCreateResponse> {
        self.post_json(route::USER, &body)
            .and_then(Client::response_json::<UserCreateResponse>)
    }

    /// User read request.
    pub fn user_read(&self, id: Uuid) -> ClientResult<UserReadResponse> {
        let route = route::user_id(id);
        self.get(&route)
            .and_then(Client::response_json::<UserReadResponse>)
    }

    /// User update request.
    pub fn user_update(&self, id: Uuid, body: UserUpdateBody) -> ClientResult<UserReadResponse> {
        let route = route::user_id(id);
        self.patch(&route, &body)
            .and_then(Client::response_json::<UserReadResponse>)
    }

    /// User delete request.
    pub fn user_delete(&self, id: Uuid) -> ClientResult<()> {
        let route = route::user_id(id);
        self.delete(&route).and_then(Client::response_empty)
    }

    fn get(&self, route: &str) -> ClientResult<Response> {
        let url = Client::url(self.url(), route)?;
        self.options
            .request_headers(self.client.get(url))
            .send()
            .map_err(Into::into)
    }

    fn get_query<T: Serialize>(&self, route: &str, query: T) -> ClientResult<Response> {
        let url = Client::url_query(self.url(), route, query)?;
        self.options
            .request_headers(self.client.get(url))
            .send()
            .map_err(Into::into)
    }

    fn post(&self, route: &str) -> ClientResult<Response> {
        let url = Client::url(self.url(), route)?;
        self.options
            .request_headers(self.client.post(url))
            .send()
            .map_err(Into::into)
    }

    fn post_json<T: Serialize>(&self, route: &str, body: &T) -> ClientResult<Response> {
        let url = Client::url(self.url(), route)?;
        self.options
            .request_headers(self.client.post(url))
            .json(body)
            .send()
            .map_err(Into::into)
    }

    fn patch<T: Serialize>(&self, route: &str, body: &T) -> ClientResult<Response> {
        let url = Client::url(self.url(), route)?;
        self.options
            .request_headers(self.client.patch(url))
            .json(body)
            .send()
            .map_err(Into::into)
    }

    fn delete(&self, route: &str) -> ClientResult<Response> {
        let url = Client::url(self.url(), route)?;
        self.options
            .request_headers(self.client.delete(url))
            .send()
            .map_err(Into::into)
    }
}

impl ClientSync {
    /// Authenticate user using token or key, returns user if successful.
    pub fn authenticate(
        &self,
        key_or_token: Option<String>,
        audit: Option<AuditDataRequest>,
    ) -> ClientResult<User> {
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
                    _ => Err(ClientError::Forbidden),
                }
                .and_then(|user_id| self.user_read(user_id))
                .map(|res| res.data)
            }
            None => Err(ClientError::Forbidden),
        }
    }
}
