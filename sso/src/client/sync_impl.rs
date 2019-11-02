use crate::{api, Client, ClientActorOptions, ClientError, ClientOptions, ClientResult, User};
use reqwest::{Client as ReqwestClient, Response};
use serde::ser::Serialize;
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

/// Client (Synchronous).
#[derive(Clone)]
pub struct ClientSync {
    url: String,
    options: ClientOptions,
    client: ReqwestClient,
}

impl fmt::Debug for ClientSync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientSync {{ url: {}, options: {:?}, client }}",
            self.url, self.options
        )
    }
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

    /// Clone client with forwarded.
    pub fn with_forwarded<F>(&self, forwarded: F) -> Self
    where
        F: Into<String>,
    {
        let mut options = self.options.clone();
        options.forwarded = forwarded.into();
        self.with_options(options)
    }

    /// Clone client with user authorisation.
    pub fn with_user_authorisation<U>(&self, user_authorisation: U) -> Self
    where
        U: Into<String>,
    {
        let mut options = self.options.clone();
        options.user_authorisation = Some(user_authorisation.into());
        self.with_options(options)
    }
}

impl ClientSync {
    /// Ping request.
    pub fn ping(&self) -> ClientResult<Value> {
        self.get(api::route::PING)
            .and_then(Client::response_json::<Value>)
    }

    /// Metrics request.
    pub fn metrics(&self) -> ClientResult<String> {
        self.get(api::route::METRICS)
            .and_then(Client::response_text)
    }

    /// Authentication local provider login request.
    pub fn auth_local_login(
        &self,
        body: api::AuthLoginRequest,
    ) -> ClientResult<api::AuthLoginResponse> {
        self.post_json(api::route::AUTH_LOCAL_LOGIN, &body)
            .and_then(Client::response_json::<api::AuthLoginResponse>)
    }

    /// Authentication local provider register request.
    pub fn auth_local_register(&self, body: api::AuthRegisterRequest) -> ClientResult<()> {
        self.post_json(api::route::AUTH_LOCAL_REGISTER, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication local provider register confirm request.
    pub fn auth_local_register_confirm(
        &self,
        body: api::AuthRegisterConfirmRequest,
    ) -> ClientResult<api::AuthPasswordMetaResponse> {
        self.post_json(api::route::AUTH_LOCAL_REGISTER_CONFIRM, &body)
            .and_then(Client::response_json::<api::AuthPasswordMetaResponse>)
    }

    /// Authentication local provider reset password request.
    pub fn auth_local_reset_password(
        &self,
        body: api::AuthResetPasswordRequest,
    ) -> ClientResult<()> {
        self.post_json(api::route::AUTH_LOCAL_RESET_PASSWORD, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication local provider reset password confirm request.
    pub fn auth_local_reset_password_confirm(
        &self,
        body: api::AuthResetPasswordConfirmRequest,
    ) -> ClientResult<api::AuthPasswordMetaResponse> {
        self.post_json(api::route::AUTH_LOCAL_RESET_PASSWORD_CONFIRM, &body)
            .and_then(Client::response_json::<api::AuthPasswordMetaResponse>)
    }

    /// Authentication local provider update email request.
    pub fn auth_local_update_email(&self, body: api::AuthUpdateEmailRequest) -> ClientResult<()> {
        self.post_json(api::route::AUTH_LOCAL_UPDATE_EMAIL, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication local provider update email revoke request.
    pub fn auth_local_update_email_revoke(&self, body: api::AuthTokenRequest) -> ClientResult<()> {
        self.post_json(api::route::AUTH_LOCAL_UPDATE_EMAIL_REVOKE, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication local provider update password request.
    pub fn auth_local_update_password(
        &self,
        body: api::AuthUpdatePasswordRequest,
    ) -> ClientResult<()> {
        self.post_json(api::route::AUTH_LOCAL_UPDATE_PASSWORD, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication local provider update password revoke request.
    pub fn auth_local_update_password_revoke(
        &self,
        body: api::AuthTokenRequest,
    ) -> ClientResult<()> {
        self.post_json(api::route::AUTH_LOCAL_UPDATE_PASSWORD_REVOKE, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication GitHub provider OAuth2 url.
    pub fn auth_github_oauth2_url(&self) -> ClientResult<api::AuthOauth2UrlResponse> {
        self.get(api::route::AUTH_GITHUB_OAUTH2)
            .and_then(Client::response_json::<api::AuthOauth2UrlResponse>)
    }

    /// Authentication GitHub provider OAuth2 url.
    pub fn auth_github_oauth2_callback(
        &self,
        body: api::AuthOauth2CallbackRequest,
    ) -> ClientResult<api::AuthTokenResponse> {
        self.post_json(api::route::AUTH_GITHUB_OAUTH2, &body)
            .and_then(Client::response_json::<api::AuthTokenResponse>)
    }

    /// Authentication Microsoft provider OAuth2 url.
    pub fn auth_microsoft_oauth2_url(&self) -> ClientResult<api::AuthOauth2UrlResponse> {
        self.get(api::route::AUTH_MICROSOFT_OAUTH2)
            .and_then(Client::response_json::<api::AuthOauth2UrlResponse>)
    }

    /// Authentication Microsoft provider OAuth2 callback.
    pub fn auth_microsoft_oauth2_callback(
        &self,
        body: api::AuthOauth2CallbackRequest,
    ) -> ClientResult<api::AuthTokenResponse> {
        self.post_json(api::route::AUTH_MICROSOFT_OAUTH2, &body)
            .and_then(Client::response_json::<api::AuthTokenResponse>)
    }

    /// Authentication verify key.
    pub fn auth_key_verify(&self, body: api::AuthKeyRequest) -> ClientResult<api::AuthKeyResponse> {
        self.post_json(api::route::AUTH_KEY_VERIFY, &body)
            .and_then(Client::response_json::<api::AuthKeyResponse>)
    }

    /// Authentication revoke key.
    pub fn auth_key_revoke(&self, body: api::AuthKeyRequest) -> ClientResult<()> {
        self.post_json(api::route::AUTH_KEY_REVOKE, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication revoke token.
    pub fn auth_token_verify(
        &self,
        body: api::AuthTokenRequest,
    ) -> ClientResult<api::AuthTokenAccessResponse> {
        self.post_json(api::route::AUTH_TOKEN_VERIFY, &body)
            .and_then(Client::response_json::<api::AuthTokenAccessResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_refresh(
        &self,
        body: api::AuthTokenRequest,
    ) -> ClientResult<api::AuthTokenResponse> {
        self.post_json(api::route::AUTH_TOKEN_REFRESH, &body)
            .and_then(Client::response_json::<api::AuthTokenResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_revoke(&self, body: api::AuthTokenRequest) -> ClientResult<()> {
        self.post_json(api::route::AUTH_TOKEN_REVOKE, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication TOTP.
    pub fn auth_totp(&self, body: api::AuthTotpRequest) -> ClientResult<()> {
        self.post_json(api::route::AUTH_TOTP, &body)
            .and_then(Client::response_empty)
    }

    /// Authentication create CSRF.
    pub fn auth_csrf_create(
        &self,
        query: api::AuthCsrfCreateRequest,
    ) -> ClientResult<api::AuthCsrfCreateResponse> {
        self.get_query(api::route::AUTH_CSRF, &query)
            .and_then(Client::response_json::<api::AuthCsrfCreateResponse>)
    }

    /// Authentication verify CSRF.
    pub fn auth_csrf_verify(&self, body: api::AuthCsrfVerifyRequest) -> ClientResult<()> {
        self.post_json(api::route::AUTH_CSRF, &body)
            .and_then(Client::response_empty)
    }

    /// Audit list request.
    pub fn audit_list(&self, query: api::AuditListRequest) -> ClientResult<api::AuditListResponse> {
        self.get_query(api::route::AUDIT, &query)
            .and_then(Client::response_json::<api::AuditListResponse>)
    }

    /// Audit create request.
    pub fn audit_create(
        &self,
        body: api::AuditCreateRequest,
    ) -> ClientResult<api::AuditReadResponse> {
        self.post_json(api::route::AUDIT, &body)
            .and_then(Client::response_json::<api::AuditReadResponse>)
    }

    /// Audit read request.
    pub fn audit_read(&self, id: Uuid) -> ClientResult<api::AuditReadResponse> {
        let route = api::route::audit_id(id);
        self.get(&route)
            .and_then(Client::response_json::<api::AuditReadResponse>)
    }

    /// Audit update request.
    pub fn audit_update(
        &self,
        id: Uuid,
        body: api::AuditUpdateRequest,
    ) -> ClientResult<api::AuditReadResponse> {
        let route = api::route::audit_id(id);
        self.patch(&route, &body)
            .and_then(Client::response_json::<api::AuditReadResponse>)
    }

    /// Key list request.
    pub fn key_list(&self, query: api::KeyListRequest) -> ClientResult<api::KeyListResponse> {
        self.get_query(api::route::KEY, &query)
            .and_then(Client::response_json::<api::KeyListResponse>)
    }

    /// Key create request.
    pub fn key_create(&self, body: api::KeyCreateRequest) -> ClientResult<api::KeyCreateResponse> {
        self.post_json(api::route::KEY, &body)
            .and_then(Client::response_json::<api::KeyCreateResponse>)
    }

    /// Key read request.
    pub fn key_read(
        &self,
        id: Uuid,
        query: api::KeyReadRequest,
    ) -> ClientResult<api::KeyReadResponse> {
        let route = api::route::key_id(id);
        self.get_query(&route, &query)
            .and_then(Client::response_json::<api::KeyReadResponse>)
    }

    /// Key update request.
    pub fn key_update(
        &self,
        id: Uuid,
        body: api::KeyUpdateRequest,
    ) -> ClientResult<api::KeyReadResponse> {
        let route = api::route::key_id(id);
        self.patch(&route, &body)
            .and_then(Client::response_json::<api::KeyReadResponse>)
    }

    /// Key delete request.
    pub fn key_delete(&self, id: Uuid) -> ClientResult<()> {
        let route = api::route::key_id(id);
        self.delete(&route).and_then(Client::response_empty)
    }

    /// Service list request.
    pub fn service_list(
        &self,
        query: api::ServiceListRequest,
    ) -> ClientResult<api::ServiceListResponse> {
        self.get_query(api::route::SERVICE, &query)
            .and_then(Client::response_json::<api::ServiceListResponse>)
    }

    /// Service create request.
    pub fn service_create(
        &self,
        body: api::ServiceCreateRequest,
    ) -> ClientResult<api::ServiceReadResponse> {
        self.post_json(api::route::SERVICE, &body)
            .and_then(Client::response_json::<api::ServiceReadResponse>)
    }

    /// Service read request.
    pub fn service_read(&self, id: Uuid) -> ClientResult<api::ServiceReadResponse> {
        let route = api::route::service_id(id);
        self.get(&route)
            .and_then(Client::response_json::<api::ServiceReadResponse>)
    }

    /// Service update request.
    pub fn service_update(
        &self,
        id: Uuid,
        body: api::ServiceUpdateRequest,
    ) -> ClientResult<api::ServiceReadResponse> {
        let route = api::route::service_id(id);
        self.patch(&route, &body)
            .and_then(Client::response_json::<api::ServiceReadResponse>)
    }

    /// Service delete request.
    pub fn service_delete(&self, id: Uuid) -> ClientResult<()> {
        let route = api::route::service_id(id);
        self.delete(&route).and_then(Client::response_empty)
    }

    /// User list request.
    pub fn user_list(&self, query: api::UserListRequest) -> ClientResult<api::UserListResponse> {
        self.get_query(api::route::USER, &query)
            .and_then(Client::response_json::<api::UserListResponse>)
    }

    /// User create request.
    pub fn user_create(
        &self,
        body: api::UserCreateRequest,
    ) -> ClientResult<api::UserCreateResponse> {
        self.post_json(api::route::USER, &body)
            .and_then(Client::response_json::<api::UserCreateResponse>)
    }

    /// User read request.
    pub fn user_read(&self, id: Uuid) -> ClientResult<api::UserReadResponse> {
        let route = api::route::user_id(id);
        self.get(&route)
            .and_then(Client::response_json::<api::UserReadResponse>)
    }

    /// User update request.
    pub fn user_update(
        &self,
        id: Uuid,
        body: api::UserUpdateRequest,
    ) -> ClientResult<api::UserReadResponse> {
        let route = api::route::user_id(id);
        self.patch(&route, &body)
            .and_then(Client::response_json::<api::UserReadResponse>)
    }

    /// User delete request.
    pub fn user_delete(&self, id: Uuid) -> ClientResult<()> {
        let route = api::route::user_id(id);
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
        audit: Option<String>,
    ) -> ClientResult<(User, Option<Uuid>)> {
        match key_or_token {
            Some(key_or_token) => {
                let (type_, value) = Client::authorisation_type(key_or_token)?;
                match type_.as_ref() {
                    "key" => {
                        let body = api::AuthKeyRequest::new(value, audit);
                        self.auth_key_verify(body)
                            .map(|res| (res.data.user, res.audit))
                    }
                    "token" => {
                        let body = api::AuthTokenRequest::new(value, audit);
                        self.auth_token_verify(body)
                            .map(|res| (res.data.user, res.audit))
                    }
                    _ => Err(ClientError::Unauthorised),
                }
            }
            None => Err(ClientError::Unauthorised),
        }
    }
}
