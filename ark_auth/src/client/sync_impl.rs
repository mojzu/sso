//! # Synchronous Client
use crate::client::{Client, ClientOptions, Error, RequestError};
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
use http::StatusCode;
use reqwest::{Client as ReqwestClient, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_json::Value;

/// Synchronous client handle.
pub struct SyncClient {
    pub options: ClientOptions,
    pub client: ReqwestClient,
}

impl SyncClient {
    /// Ping request.
    pub fn ping(&self) -> Result<Value, Error> {
        SyncClient::send_response_json(self.get(route::PING))
    }

    /// Metrics request.
    pub fn metrics(&self) -> Result<String, Error> {
        SyncClient::send_response_text(self.get(route::METRICS))
    }

    /// Authenticate user using token or key, returns user if successful.
    pub fn authenticate(
        &self,
        key_or_token: Option<String>,
        audit: Option<AuditDataRequest>,
    ) -> Result<User, Error> {
        match key_or_token {
            Some(key_or_token) => {
                let (type_, value) = ClientOptions::split_authorisation(key_or_token)?;
                match type_.as_ref() {
                    "key" => {
                        let body = AuthKeyBody::new(value, audit);
                        self.auth_key_verify(body).map(|res| res.data.user_id)
                    }
                    "token" => {
                        let body = AuthTokenBody::new(value, audit);
                        self.auth_token_verify(body).map(|res| res.data.user_id)
                    }
                    _ => Err(Error::InvalidKeyOrToken),
                }
                .and_then(|user_id| self.user_read(&user_id))
                .map(|res| res.data)
            }
            None => Err(Error::InvalidKeyOrToken),
        }
    }

    /// Authentication local provider login request.
    pub fn auth_local_login(&self, body: AuthLoginBody) -> Result<AuthLoginResponse, Error> {
        SyncClient::send_response_json::<AuthLoginResponse>(
            self.post(route::AUTH_LOCAL_LOGIN).json(&body),
        )
    }

    /// Authentication local provider reset password request.
    pub fn auth_local_reset_password(&self, body: AuthResetPasswordBody) -> Result<(), Error> {
        SyncClient::send_response_empty(self.post(route::AUTH_LOCAL_RESET_PASSWORD).json(&body))
    }

    /// Authentication local provider reset password confirm request.
    pub fn auth_local_reset_password_confirm(
        &self,
        body: AuthResetPasswordConfirmBody,
    ) -> Result<AuthPasswordMetaResponse, Error> {
        SyncClient::send_response_json::<AuthPasswordMetaResponse>(
            self.post(route::AUTH_LOCAL_RESET_PASSWORD_CONFIRM)
                .json(&body),
        )
    }

    /// Authentication local provider update email request.
    pub fn auth_local_update_email(&self, body: AuthUpdateEmailBody) -> Result<(), Error> {
        SyncClient::send_response_empty(self.post(route::AUTH_LOCAL_UPDATE_EMAIL).json(&body))
    }

    /// Authentication local provider update email revoke request.
    pub fn auth_local_update_email_revoke(&self, body: AuthTokenBody) -> Result<(), Error> {
        SyncClient::send_response_empty(
            self.post(route::AUTH_LOCAL_UPDATE_EMAIL_REVOKE).json(&body),
        )
    }

    /// Authentication local provider update password request.
    pub fn auth_local_update_password(&self, body: AuthUpdatePasswordBody) -> Result<(), Error> {
        SyncClient::send_response_empty(self.post(route::AUTH_LOCAL_UPDATE_PASSWORD).json(&body))
    }

    /// Authentication local provider update password revoke request.
    pub fn auth_local_update_password_revoke(&self, body: AuthTokenBody) -> Result<(), Error> {
        SyncClient::send_response_empty(
            self.post(route::AUTH_LOCAL_UPDATE_PASSWORD_REVOKE)
                .json(&body),
        )
    }

    /// Authentication GitHub provider OAuth2 request.
    pub fn auth_github_oauth2_request(&self) -> Result<AuthOauth2UrlResponse, Error> {
        SyncClient::send_response_json::<AuthOauth2UrlResponse>(
            self.post(route::AUTH_GITHUB_OAUTH2),
        )
    }

    /// Authentication Microsoft provider OAuth2 request.
    pub fn auth_microsoft_oauth2_request(&self) -> Result<AuthOauth2UrlResponse, Error> {
        SyncClient::send_response_json::<AuthOauth2UrlResponse>(
            self.post(route::AUTH_MICROSOFT_OAUTH2),
        )
    }

    /// Authentication verify key.
    pub fn auth_key_verify(&self, body: AuthKeyBody) -> Result<AuthKeyResponse, Error> {
        SyncClient::send_response_json::<AuthKeyResponse>(
            self.post(route::AUTH_KEY_VERIFY).json(&body),
        )
    }

    /// Authentication revoke key.
    pub fn auth_key_revoke(&self, body: AuthKeyBody) -> Result<(), Error> {
        SyncClient::send_response_empty(self.post(route::AUTH_KEY_REVOKE).json(&body))
    }

    /// Authentication revoke token.
    pub fn auth_token_verify(
        &self,
        body: AuthTokenBody,
    ) -> Result<AuthTokenPartialResponse, Error> {
        SyncClient::send_response_json::<AuthTokenPartialResponse>(
            self.post(route::AUTH_TOKEN_VERIFY).json(&body),
        )
    }

    /// Authentication revoke token.
    pub fn auth_token_refresh(&self, body: AuthTokenBody) -> Result<AuthTokenResponse, Error> {
        SyncClient::send_response_json::<AuthTokenResponse>(
            self.post(route::AUTH_TOKEN_REFRESH).json(&body),
        )
    }

    /// Authentication revoke token.
    pub fn auth_token_revoke(&self, body: AuthTokenBody) -> Result<(), Error> {
        SyncClient::send_response_empty(self.post(route::AUTH_TOKEN_REVOKE).json(&body))
    }

    /// Audit list request.
    pub fn audit_list(&self, query: AuditListQuery) -> Result<AuditListResponse, Error> {
        SyncClient::send_response_json::<AuditListResponse>(self.get_query(route::AUDIT, query))
    }

    /// Audit create request.
    pub fn audit_create(&self, body: AuditCreateBody) -> Result<AuditReadResponse, Error> {
        SyncClient::send_response_json::<AuditReadResponse>(self.post(route::AUDIT).json(&body))
    }

    /// Audit read by ID request.
    pub fn audit_read(&self, id: &str) -> Result<AuditReadResponse, Error> {
        let path = route::audit_id(id);
        SyncClient::send_response_json::<AuditReadResponse>(self.get(&path))
    }

    /// Key list request.
    pub fn key_list(&self, query: KeyListQuery) -> Result<KeyListResponse, Error> {
        SyncClient::send_response_json::<KeyListResponse>(self.get_query(route::KEY, query))
    }

    /// Key create request.
    pub fn key_create(&self, body: KeyCreateBody) -> Result<KeyReadResponse, Error> {
        SyncClient::send_response_json::<KeyReadResponse>(self.post(route::KEY).json(&body))
    }

    /// Key read request.
    pub fn key_read(&self, id: &str) -> Result<KeyReadResponse, Error> {
        let path = route::key_id(id);
        SyncClient::send_response_json::<KeyReadResponse>(self.get(&path))
    }

    /// Key update request.
    pub fn key_update(&self, id: &str, body: KeyUpdateBody) -> Result<KeyReadResponse, Error> {
        let path = route::key_id(id);
        SyncClient::send_response_json::<KeyReadResponse>(self.patch(&path).json(&body))
    }

    /// Key delete request.
    pub fn key_delete(&self, id: &str) -> Result<(), Error> {
        let path = route::key_id(id);
        SyncClient::send_response_empty(self.delete(&path))
    }

    /// Service list request.
    pub fn service_list(&self, query: ServiceListQuery) -> Result<ServiceListResponse, Error> {
        SyncClient::send_response_json::<ServiceListResponse>(self.get_query(route::SERVICE, query))
    }

    /// Service create request.
    pub fn service_create(&self, body: ServiceCreateBody) -> Result<ServiceReadResponse, Error> {
        SyncClient::send_response_json::<ServiceReadResponse>(self.post(route::SERVICE).json(&body))
    }

    /// Service read request.
    pub fn service_read(&self, id: &str) -> Result<ServiceReadResponse, Error> {
        let path = route::service_id(id);
        SyncClient::send_response_json::<ServiceReadResponse>(self.get(&path))
    }

    /// Service update request.
    pub fn service_update(
        &self,
        id: &str,
        body: ServiceUpdateBody,
    ) -> Result<ServiceReadResponse, Error> {
        let path = route::service_id(id);
        SyncClient::send_response_json::<ServiceReadResponse>(self.patch(&path).json(&body))
    }

    /// Service delete request.
    pub fn service_delete(&self, id: &str) -> Result<(), Error> {
        let path = route::service_id(id);
        SyncClient::send_response_empty(self.delete(&path))
    }

    /// User list request.
    pub fn user_list(&self, query: UserListQuery) -> Result<UserListResponse, Error> {
        SyncClient::send_response_json::<UserListResponse>(self.get_query(route::USER, query))
    }

    /// User create request.
    pub fn user_create(&self, body: UserCreateBody) -> Result<UserCreateResponse, Error> {
        SyncClient::send_response_json::<UserCreateResponse>(self.post(route::USER).json(&body))
    }

    /// User read request.
    pub fn user_read(&self, id: &str) -> Result<UserReadResponse, Error> {
        let path = route::user_id(id);
        SyncClient::send_response_json::<UserReadResponse>(self.get(&path))
    }

    /// User update request.
    pub fn user_update(&self, id: &str, body: UserUpdateBody) -> Result<UserReadResponse, Error> {
        let path = route::user_id(id);
        SyncClient::send_response_json::<UserReadResponse>(self.patch(&path).json(&body))
    }

    /// User delete request.
    pub fn user_delete(&self, id: &str) -> Result<(), Error> {
        let path = route::user_id(id);
        SyncClient::send_response_empty(self.delete(&path))
    }

    fn build_client(options: &ClientOptions) -> ReqwestClient {
        let headers = options.default_headers();
        let builder = reqwest::ClientBuilder::new()
            .use_rustls_tls()
            .default_headers(headers);

        // Optional CA and client certificates.
        let builder = match &options.crt_pem {
            Some(buf) => {
                let crt_pem = reqwest::Certificate::from_pem(buf).unwrap();
                builder.add_root_certificate(crt_pem)
            }
            None => builder,
        };
        let builder = match &options.client_pem {
            Some(buf) => {
                let client_pem = reqwest::Identity::from_pem(buf).unwrap();
                builder.identity(client_pem)
            }
            None => builder,
        };

        builder.build().unwrap()
    }

    fn get(&self, path: &str) -> RequestBuilder {
        let url = self.options.url_path(path).unwrap();
        self.client.get(url)
    }

    fn get_query<T: Serialize>(&self, path: &str, query: T) -> RequestBuilder {
        let url = self.options.url_path_query(path, query).unwrap();
        self.client.get(url)
    }

    fn post(&self, path: &str) -> RequestBuilder {
        let url = self.options.url_path(path).unwrap();
        self.client.post(url)
    }

    fn patch(&self, path: &str) -> RequestBuilder {
        let url = self.options.url_path(path).unwrap();
        self.client.patch(url)
    }

    fn delete(&self, path: &str) -> RequestBuilder {
        let url = self.options.url_path(path).unwrap();
        self.client.delete(url)
    }

    fn send(request: RequestBuilder) -> Result<Response, Error> {
        request
            .send()
            .map_err(Into::into)
            .and_then(|response| match response.status() {
                StatusCode::OK => Ok(response),
                StatusCode::BAD_REQUEST => Err(Error::Request(RequestError::BadRequest)),
                StatusCode::FORBIDDEN => Err(Error::Request(RequestError::Forbidden)),
                StatusCode::NOT_FOUND => Err(Error::Request(RequestError::NotFound)),
                _ => Err(Error::Response),
            })
    }

    fn send_response_empty(request: RequestBuilder) -> Result<(), Error> {
        SyncClient::send(request).map(|_| ())
    }

    fn send_response_json<T: DeserializeOwned>(request: RequestBuilder) -> Result<T, Error> {
        SyncClient::send(request).and_then(|mut res| res.json::<T>().map_err(Into::into))
    }

    fn send_response_text(request: RequestBuilder) -> Result<String, Error> {
        SyncClient::send(request).and_then(|mut res| res.text().map_err(Into::into))
    }
}

impl Client for SyncClient {
    fn new(options: ClientOptions) -> Self {
        let client = SyncClient::build_client(&options);
        SyncClient { options, client }
    }
}
