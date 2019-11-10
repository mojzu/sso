use crate::{
    api::{self, ApiError, ApiResult},
    ClientOptions, ClientRequestOptions, DriverError, DriverResult, User,
};
use reqwest::{Client as ReqwestClient, Response};
use serde::{de::DeserializeOwned, ser::Serialize};
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

/// Client (Synchronous).
#[derive(Clone)]
pub struct ClientSync {
    url: String,
    client: ReqwestClient,
    options: ClientRequestOptions,
}

impl fmt::Debug for ClientSync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientSync {{ url: {}, client, options: {:?} }}",
            self.url, self.options
        )
    }
}

impl ClientSync {
    /// Create new client.
    pub fn new<U>(url: U, options: ClientOptions) -> DriverResult<Self>
    where
        U: Into<String>,
    {
        let headers = options.default_headers();
        let builder = reqwest::ClientBuilder::new()
            .use_rustls_tls()
            .default_headers(headers);
        let builder = match options.crt_pem() {
            Some(buf) => {
                let crt_pem = reqwest::Certificate::from_pem(buf).unwrap();
                builder.add_root_certificate(crt_pem)
            }
            None => builder,
        };
        let builder = match options.client_pem() {
            Some(buf) => {
                let client_pem = reqwest::Identity::from_pem(buf).unwrap();
                builder.identity(client_pem)
            }
            None => builder,
        };
        let client = builder.build().unwrap();

        Ok(Self {
            url: url.into(),
            client,
            options: options.request,
        })
    }

    /// Returns url reference.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns client reference.
    pub fn client(&self) -> &ReqwestClient {
        &self.client
    }

    /// Returns options reference.
    pub fn options(&self) -> &ClientRequestOptions {
        &self.options
    }

    /// Clone client with request options.
    pub fn with_options(&self, options: ClientRequestOptions) -> Self {
        Self {
            url: self.url.clone(),
            client: self.client.clone(),
            options,
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
    pub fn ping(&self) -> ApiResult<Value> {
        self.get(api::route::PING)
            .and_then(Self::response_json::<Value>)
    }

    /// Metrics request.
    pub fn metrics(&self) -> ApiResult<String> {
        self.get(api::route::METRICS).and_then(Self::response_text)
    }

    /// Authentication local provider login request.
    pub fn auth_local_login(
        &self,
        body: api::AuthLoginRequest,
    ) -> ApiResult<api::AuthLoginResponse> {
        self.post(api::route::AUTH_LOCAL_LOGIN, &body)
            .and_then(Self::response_json::<api::AuthLoginResponse>)
    }

    /// Authentication local provider register request.
    pub fn auth_local_register(&self, body: api::AuthRegisterRequest) -> ApiResult<()> {
        self.post(api::route::AUTH_LOCAL_REGISTER, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication local provider register confirm request.
    pub fn auth_local_register_confirm(
        &self,
        body: api::AuthRegisterConfirmRequest,
    ) -> ApiResult<api::AuthPasswordMetaResponse> {
        self.post(api::route::AUTH_LOCAL_REGISTER_CONFIRM, &body)
            .and_then(Self::response_json::<api::AuthPasswordMetaResponse>)
    }

    /// Authentication local provider register revoke request.
    pub fn auth_local_register_revoke(&self, body: api::AuthTokenRequest) -> ApiResult<()> {
        self.post(api::route::AUTH_LOCAL_REGISTER_REVOKE, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication local provider reset password request.
    pub fn auth_local_reset_password(&self, body: api::AuthResetPasswordRequest) -> ApiResult<()> {
        self.post(api::route::AUTH_LOCAL_RESET_PASSWORD, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication local provider reset password confirm request.
    pub fn auth_local_reset_password_confirm(
        &self,
        body: api::AuthResetPasswordConfirmRequest,
    ) -> ApiResult<api::AuthPasswordMetaResponse> {
        self.post(api::route::AUTH_LOCAL_RESET_PASSWORD_CONFIRM, &body)
            .and_then(Self::response_json::<api::AuthPasswordMetaResponse>)
    }

    /// Authentication local provider reset password revoke request.
    pub fn auth_local_reset_password_revoke(&self, body: api::AuthTokenRequest) -> ApiResult<()> {
        self.post(api::route::AUTH_LOCAL_RESET_PASSWORD_REVOKE, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication local provider update email request.
    pub fn auth_local_update_email(&self, body: api::AuthUpdateEmailRequest) -> ApiResult<()> {
        self.post(api::route::AUTH_LOCAL_UPDATE_EMAIL, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication local provider update email revoke request.
    pub fn auth_local_update_email_revoke(&self, body: api::AuthTokenRequest) -> ApiResult<()> {
        self.post(api::route::AUTH_LOCAL_UPDATE_EMAIL_REVOKE, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication local provider update password request.
    pub fn auth_local_update_password(
        &self,
        body: api::AuthUpdatePasswordRequest,
    ) -> ApiResult<()> {
        self.post(api::route::AUTH_LOCAL_UPDATE_PASSWORD, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication local provider update password revoke request.
    pub fn auth_local_update_password_revoke(&self, body: api::AuthTokenRequest) -> ApiResult<()> {
        self.post(api::route::AUTH_LOCAL_UPDATE_PASSWORD_REVOKE, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication GitHub provider OAuth2 url.
    pub fn auth_github_oauth2_url(&self) -> ApiResult<api::AuthOauth2UrlResponse> {
        self.get(api::route::AUTH_GITHUB_OAUTH2)
            .and_then(Self::response_json::<api::AuthOauth2UrlResponse>)
    }

    /// Authentication GitHub provider OAuth2 url.
    pub fn auth_github_oauth2_callback(
        &self,
        body: api::AuthOauth2CallbackRequest,
    ) -> ApiResult<api::AuthTokenResponse> {
        self.post(api::route::AUTH_GITHUB_OAUTH2, &body)
            .and_then(Self::response_json::<api::AuthTokenResponse>)
    }

    /// Authentication Microsoft provider OAuth2 url.
    pub fn auth_microsoft_oauth2_url(&self) -> ApiResult<api::AuthOauth2UrlResponse> {
        self.get(api::route::AUTH_MICROSOFT_OAUTH2)
            .and_then(Self::response_json::<api::AuthOauth2UrlResponse>)
    }

    /// Authentication Microsoft provider OAuth2 callback.
    pub fn auth_microsoft_oauth2_callback(
        &self,
        body: api::AuthOauth2CallbackRequest,
    ) -> ApiResult<api::AuthTokenResponse> {
        self.post(api::route::AUTH_MICROSOFT_OAUTH2, &body)
            .and_then(Self::response_json::<api::AuthTokenResponse>)
    }

    /// Authentication verify key.
    pub fn auth_key_verify(&self, body: api::AuthKeyRequest) -> ApiResult<api::AuthKeyResponse> {
        self.post(api::route::AUTH_KEY_VERIFY, &body)
            .and_then(Self::response_json::<api::AuthKeyResponse>)
    }

    /// Authentication revoke key.
    pub fn auth_key_revoke(&self, body: api::AuthKeyRequest) -> ApiResult<()> {
        self.post(api::route::AUTH_KEY_REVOKE, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication revoke token.
    pub fn auth_token_verify(
        &self,
        body: api::AuthTokenRequest,
    ) -> ApiResult<api::AuthTokenAccessResponse> {
        self.post(api::route::AUTH_TOKEN_VERIFY, &body)
            .and_then(Self::response_json::<api::AuthTokenAccessResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_refresh(
        &self,
        body: api::AuthTokenRequest,
    ) -> ApiResult<api::AuthTokenResponse> {
        self.post(api::route::AUTH_TOKEN_REFRESH, &body)
            .and_then(Self::response_json::<api::AuthTokenResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_revoke(&self, body: api::AuthTokenRequest) -> ApiResult<()> {
        self.post(api::route::AUTH_TOKEN_REVOKE, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication TOTP.
    pub fn auth_totp(&self, body: api::AuthTotpRequest) -> ApiResult<()> {
        self.post(api::route::AUTH_TOTP, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication create CSRF.
    pub fn auth_csrf_create(
        &self,
        query: api::AuthCsrfCreateRequest,
    ) -> ApiResult<api::AuthCsrfCreateResponse> {
        self.get_query(api::route::AUTH_CSRF, &query)
            .and_then(Self::response_json::<api::AuthCsrfCreateResponse>)
    }

    /// Authentication verify CSRF.
    pub fn auth_csrf_verify(&self, body: api::AuthCsrfVerifyRequest) -> ApiResult<()> {
        self.post(api::route::AUTH_CSRF, &body)
            .and_then(Self::response_empty)
    }

    /// Audit list request.
    pub fn audit_list(&self, query: api::AuditListRequest) -> ApiResult<api::AuditListResponse> {
        self.get_query(api::route::AUDIT, &query)
            .and_then(Self::response_json::<api::AuditListResponse>)
    }

    /// Audit create request.
    pub fn audit_create(&self, body: api::AuditCreateRequest) -> ApiResult<api::AuditReadResponse> {
        self.post(api::route::AUDIT, &body)
            .and_then(Self::response_json::<api::AuditReadResponse>)
    }

    /// Audit read request.
    pub fn audit_read(
        &self,
        id: Uuid,
        query: api::AuditReadRequest,
    ) -> ApiResult<api::AuditReadResponse> {
        let route = api::route::audit_id(id);
        self.get_query(&route, &query)
            .and_then(Self::response_json::<api::AuditReadResponse>)
    }

    /// Audit update request.
    pub fn audit_update(
        &self,
        id: Uuid,
        body: api::AuditUpdateRequest,
    ) -> ApiResult<api::AuditReadResponse> {
        let route = api::route::audit_id(id);
        self.patch(&route, &body)
            .and_then(Self::response_json::<api::AuditReadResponse>)
    }

    /// Key list request.
    pub fn key_list(&self, query: api::KeyListRequest) -> ApiResult<api::KeyListResponse> {
        self.get_query(api::route::KEY, &query)
            .and_then(Self::response_json::<api::KeyListResponse>)
    }

    /// Key create request.
    pub fn key_create(&self, body: api::KeyCreateRequest) -> ApiResult<api::KeyCreateResponse> {
        self.post(api::route::KEY, &body)
            .and_then(Self::response_json::<api::KeyCreateResponse>)
    }

    /// Key read request.
    pub fn key_read(
        &self,
        id: Uuid,
        query: api::KeyReadRequest,
    ) -> ApiResult<api::KeyReadResponse> {
        let route = api::route::key_id(id);
        self.get_query(&route, &query)
            .and_then(Self::response_json::<api::KeyReadResponse>)
    }

    /// Key update request.
    pub fn key_update(
        &self,
        id: Uuid,
        body: api::KeyUpdateRequest,
    ) -> ApiResult<api::KeyReadResponse> {
        let route = api::route::key_id(id);
        self.patch(&route, &body)
            .and_then(Self::response_json::<api::KeyReadResponse>)
    }

    /// Key delete request.
    pub fn key_delete(&self, id: Uuid) -> ApiResult<()> {
        let route = api::route::key_id(id);
        self.delete(&route).and_then(Self::response_empty)
    }

    /// Service list request.
    pub fn service_list(
        &self,
        query: api::ServiceListRequest,
    ) -> ApiResult<api::ServiceListResponse> {
        self.get_query(api::route::SERVICE, &query)
            .and_then(Self::response_json::<api::ServiceListResponse>)
    }

    /// Service create request.
    pub fn service_create(
        &self,
        body: api::ServiceCreateRequest,
    ) -> ApiResult<api::ServiceReadResponse> {
        self.post(api::route::SERVICE, &body)
            .and_then(Self::response_json::<api::ServiceReadResponse>)
    }

    /// Service read request.
    pub fn service_read(&self, id: Uuid) -> ApiResult<api::ServiceReadResponse> {
        let route = api::route::service_id(id);
        self.get(&route)
            .and_then(Self::response_json::<api::ServiceReadResponse>)
    }

    /// Service update request.
    pub fn service_update(
        &self,
        id: Uuid,
        body: api::ServiceUpdateRequest,
    ) -> ApiResult<api::ServiceReadResponse> {
        let route = api::route::service_id(id);
        self.patch(&route, &body)
            .and_then(Self::response_json::<api::ServiceReadResponse>)
    }

    /// Service delete request.
    pub fn service_delete(&self, id: Uuid) -> ApiResult<()> {
        let route = api::route::service_id(id);
        self.delete(&route).and_then(Self::response_empty)
    }

    /// User list request.
    pub fn user_list(&self, query: api::UserListRequest) -> ApiResult<api::UserListResponse> {
        self.get_query(api::route::USER, &query)
            .and_then(Self::response_json::<api::UserListResponse>)
    }

    /// User create request.
    pub fn user_create(&self, body: api::UserCreateRequest) -> ApiResult<api::UserCreateResponse> {
        self.post(api::route::USER, &body)
            .and_then(Self::response_json::<api::UserCreateResponse>)
    }

    /// User read request.
    pub fn user_read(&self, id: Uuid) -> ApiResult<api::UserReadResponse> {
        let route = api::route::user_id(id);
        self.get(&route)
            .and_then(Self::response_json::<api::UserReadResponse>)
    }

    /// User update request.
    pub fn user_update(
        &self,
        id: Uuid,
        body: api::UserUpdateRequest,
    ) -> ApiResult<api::UserReadResponse> {
        let route = api::route::user_id(id);
        self.patch(&route, &body)
            .and_then(Self::response_json::<api::UserReadResponse>)
    }

    /// User delete request.
    pub fn user_delete(&self, id: Uuid) -> ApiResult<()> {
        let route = api::route::user_id(id);
        self.delete(&route).and_then(Self::response_empty)
    }

    fn get(&self, route: &str) -> ApiResult<Response> {
        let url = ClientOptions::url(self.url(), route).unwrap();
        self.options
            .request_headers_sync(self.client.get(url))
            .send()
            .map_err(Into::into)
    }

    fn get_query<T: Serialize>(&self, route: &str, query: T) -> ApiResult<Response> {
        let url = ClientOptions::url_query(self.url(), route, query).unwrap();
        self.options
            .request_headers_sync(self.client.get(url))
            .send()
            .map_err(Into::into)
    }

    fn post<T: Serialize>(&self, route: &str, body: &T) -> ApiResult<Response> {
        let url = ClientOptions::url(self.url(), route).unwrap();
        self.options
            .request_headers_sync(self.client.post(url))
            .json(body)
            .send()
            .map_err(Into::into)
    }

    fn patch<T: Serialize>(&self, route: &str, body: &T) -> ApiResult<Response> {
        let url = ClientOptions::url(self.url(), route).unwrap();
        self.options
            .request_headers_sync(self.client.patch(url))
            .json(body)
            .send()
            .map_err(Into::into)
    }

    fn delete(&self, route: &str) -> ApiResult<Response> {
        let url = ClientOptions::url(self.url(), route).unwrap();
        self.options
            .request_headers_sync(self.client.delete(url))
            .send()
            .map_err(Into::into)
    }

    fn response_json<T: DeserializeOwned>(res: Response) -> ApiResult<T> {
        res.error_for_status()
            .map_err(Into::into)
            .and_then(|mut res| res.json::<T>())
            .map_err(Into::into)
    }

    fn response_text(res: Response) -> ApiResult<String> {
        res.error_for_status()
            .map_err(Into::into)
            .and_then(|mut res| res.text())
            .map_err(Into::into)
    }

    fn response_empty(res: Response) -> ApiResult<()> {
        res.error_for_status().map_err(Into::into).map(|_| ())
    }
}

impl ClientSync {
    /// Authenticate user using token or key, returns user if successful.
    pub fn authenticate(
        &self,
        key_or_token: Option<String>,
        audit: Option<String>,
    ) -> ApiResult<(User, Option<Uuid>)> {
        match key_or_token {
            Some(key_or_token) => {
                let (type_, value) = ClientOptions::authorisation_type(key_or_token)
                    .map_err(ApiError::Unauthorised)?;
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
                    _ => Err(ApiError::Unauthorised(DriverError::AuthenticateTypeUnknown)),
                }
            }
            None => Err(ApiError::Unauthorised(
                DriverError::AuthenticateKeyOrTokenUndefined,
            )),
        }
    }
}
