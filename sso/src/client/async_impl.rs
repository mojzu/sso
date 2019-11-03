use crate::{
    api::{self, ApiError, ApiResult},
    ClientOptions, ClientRequestOptions, DriverError, DriverResult, User,
};
use futures::future::Either;
use futures::{future, Future};
use reqwest::r#async::Client as ReqwestClient;
use serde::{de::DeserializeOwned, ser::Serialize};
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

/// Client (Asynchronous).
#[derive(Clone)]
pub struct ClientAsync {
    url: String,
    client: ReqwestClient,
    options: ClientRequestOptions,
}

impl fmt::Debug for ClientAsync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientAsync {{ url: {}, client, options: {:?} }}",
            self.url, self.options
        )
    }
}

impl ClientAsync {
    /// Create new client handle.
    pub fn new<U>(url: U, options: ClientOptions) -> DriverResult<Self>
    where
        U: Into<String>,
    {
        let headers = options.default_headers();
        let builder = reqwest::r#async::ClientBuilder::new()
            .use_rustls_tls()
            .default_headers(headers);
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

impl ClientAsync {
    /// Ping request.
    pub fn ping(&self) -> impl Future<Item = Value, Error = ApiError> {
        self.get(api::route::PING)
            .and_then(Self::response_json::<Value>)
    }

    /// Metrics request.
    pub fn metrics(&self) -> impl Future<Item = String, Error = ApiError> {
        self.get(api::route::METRICS)
    }

    /// Authentication local provider login request.
    pub fn auth_local_login(
        &self,
        body: api::AuthLoginRequest,
    ) -> impl Future<Item = api::AuthLoginResponse, Error = ApiError> {
        self.post(api::route::AUTH_LOCAL_LOGIN, &body)
            .and_then(Self::response_json::<api::AuthLoginResponse>)
    }

    /// Authentication local provider register request.
    pub fn auth_local_register(
        &self,
        body: api::AuthRegisterRequest,
    ) -> impl Future<Item = (), Error = ApiError> {
        self.post(api::route::AUTH_LOCAL_REGISTER, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication local provider register confirm request.
    pub fn auth_local_register_confirm(
        &self,
        body: api::AuthRegisterConfirmRequest,
    ) -> impl Future<Item = api::AuthPasswordMetaResponse, Error = ApiError> {
        self.post(api::route::AUTH_LOCAL_REGISTER_CONFIRM, &body)
            .and_then(Self::response_json::<api::AuthPasswordMetaResponse>)
    }

    /// Authentication local provider reset password request.
    pub fn auth_local_reset_password(
        &self,
        body: api::AuthResetPasswordRequest,
    ) -> impl Future<Item = (), Error = ApiError> {
        self.post(api::route::AUTH_LOCAL_RESET_PASSWORD, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication local provider reset password confirm request.
    pub fn auth_local_reset_password_confirm(
        &self,
        body: api::AuthResetPasswordConfirmRequest,
    ) -> impl Future<Item = api::AuthPasswordMetaResponse, Error = ApiError> {
        self.post(api::route::AUTH_LOCAL_RESET_PASSWORD_CONFIRM, &body)
            .and_then(Self::response_json::<api::AuthPasswordMetaResponse>)
    }

    /// Authentication local provider update email request.
    pub fn auth_local_update_email(
        &self,
        body: api::AuthUpdateEmailRequest,
    ) -> impl Future<Item = (), Error = ApiError> {
        self.post(api::route::AUTH_LOCAL_UPDATE_EMAIL, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication local provider update email revoke request.
    pub fn auth_local_update_email_revoke(
        &self,
        body: api::AuthTokenRequest,
    ) -> impl Future<Item = (), Error = ApiError> {
        self.post(api::route::AUTH_LOCAL_UPDATE_EMAIL_REVOKE, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication local provider update password request.
    pub fn auth_local_update_password(
        &self,
        body: api::AuthUpdatePasswordRequest,
    ) -> impl Future<Item = (), Error = ApiError> {
        self.post(api::route::AUTH_LOCAL_UPDATE_PASSWORD, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication local provider update password revoke request.
    pub fn auth_local_update_password_revoke(
        &self,
        body: api::AuthTokenRequest,
    ) -> impl Future<Item = (), Error = ApiError> {
        self.post(api::route::AUTH_LOCAL_UPDATE_PASSWORD_REVOKE, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication GitHub provider OAuth2 url.
    pub fn auth_github_oauth2_url(
        &self,
    ) -> impl Future<Item = api::AuthOauth2UrlResponse, Error = ApiError> {
        self.get(api::route::AUTH_GITHUB_OAUTH2)
            .and_then(Self::response_json::<api::AuthOauth2UrlResponse>)
    }

    /// Authentication GitHub provider OAuth2 callback.
    pub fn auth_github_oauth2_callback(
        &self,
        body: api::AuthOauth2CallbackRequest,
    ) -> impl Future<Item = api::AuthTokenResponse, Error = ApiError> {
        self.post(api::route::AUTH_GITHUB_OAUTH2, &body)
            .and_then(Self::response_json::<api::AuthTokenResponse>)
    }

    /// Authentication Microsoft provider OAuth2 url.
    pub fn auth_microsoft_oauth2_url(
        &self,
    ) -> impl Future<Item = api::AuthOauth2UrlResponse, Error = ApiError> {
        self.get(api::route::AUTH_MICROSOFT_OAUTH2)
            .and_then(Self::response_json::<api::AuthOauth2UrlResponse>)
    }

    /// Authentication Microsoft provider OAuth2 callback.
    pub fn auth_microsoft_oauth2_callback(
        &self,
        body: api::AuthOauth2CallbackRequest,
    ) -> impl Future<Item = api::AuthTokenResponse, Error = ApiError> {
        self.post(api::route::AUTH_MICROSOFT_OAUTH2, &body)
            .and_then(Self::response_json::<api::AuthTokenResponse>)
    }

    /// Authentication verify key.
    pub fn auth_key_verify(
        &self,
        body: api::AuthKeyRequest,
    ) -> impl Future<Item = api::AuthKeyResponse, Error = ApiError> {
        self.post(api::route::AUTH_KEY_VERIFY, &body)
            .and_then(Self::response_json::<api::AuthKeyResponse>)
    }

    /// Authentication revoke key.
    pub fn auth_key_revoke(
        &self,
        body: api::AuthKeyRequest,
    ) -> impl Future<Item = (), Error = ApiError> {
        self.post(api::route::AUTH_KEY_REVOKE, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication revoke token.
    pub fn auth_token_verify(
        &self,
        body: api::AuthTokenRequest,
    ) -> impl Future<Item = api::AuthTokenAccessResponse, Error = ApiError> {
        self.post(api::route::AUTH_TOKEN_VERIFY, &body)
            .and_then(Self::response_json::<api::AuthTokenAccessResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_refresh(
        &self,
        body: api::AuthTokenRequest,
    ) -> impl Future<Item = api::AuthTokenResponse, Error = ApiError> {
        self.post(api::route::AUTH_TOKEN_REFRESH, &body)
            .and_then(Self::response_json::<api::AuthTokenResponse>)
    }

    /// Authentication revoke token.
    pub fn auth_token_revoke(
        &self,
        body: api::AuthTokenRequest,
    ) -> impl Future<Item = (), Error = ApiError> {
        self.post(api::route::AUTH_TOKEN_REVOKE, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication TOTP.
    pub fn auth_totp(
        &self,
        body: api::AuthTotpRequest,
    ) -> impl Future<Item = (), Error = ApiError> {
        self.post(api::route::AUTH_TOTP, &body)
            .and_then(Self::response_empty)
    }

    /// Authentication create CSRF.
    pub fn auth_csrf_create(
        &self,
        query: api::AuthCsrfCreateRequest,
    ) -> impl Future<Item = api::AuthCsrfCreateResponse, Error = ApiError> {
        self.get_query(api::route::AUTH_CSRF, query)
            .and_then(Self::response_json::<api::AuthCsrfCreateResponse>)
    }

    /// Authentication verify CSRF.
    pub fn auth_csrf_verify(
        &self,
        body: api::AuthCsrfVerifyRequest,
    ) -> impl Future<Item = (), Error = ApiError> {
        self.post(api::route::AUTH_CSRF, &body)
            .and_then(Self::response_empty)
    }

    /// Audit list request.
    pub fn audit_list(
        &self,
        query: api::AuditListRequest,
    ) -> impl Future<Item = api::AuditListResponse, Error = ApiError> {
        self.get_query(api::route::AUDIT, query)
            .and_then(Self::response_json::<api::AuditListResponse>)
    }

    /// Audit create request.
    pub fn audit_create(
        &self,
        body: api::AuditCreateRequest,
    ) -> impl Future<Item = api::AuditReadResponse, Error = ApiError> {
        self.post(api::route::AUDIT, &body)
            .and_then(Self::response_json::<api::AuditReadResponse>)
    }

    /// Audit read request.
    pub fn audit_read(
        &self,
        id: Uuid,
    ) -> impl Future<Item = api::AuditReadResponse, Error = ApiError> {
        self.get(api::route::audit_id(id))
            .and_then(Self::response_json::<api::AuditReadResponse>)
    }

    /// Audit update request.
    pub fn audit_update(
        &self,
        id: Uuid,
        body: api::AuditUpdateRequest,
    ) -> impl Future<Item = api::AuditReadResponse, Error = ApiError> {
        self.patch(api::route::audit_id(id), &body)
            .and_then(Self::response_json::<api::AuditReadResponse>)
    }

    /// Key list request.
    pub fn key_list(
        &self,
        query: api::KeyListRequest,
    ) -> impl Future<Item = api::KeyListResponse, Error = ApiError> {
        self.get_query(api::route::KEY, query)
            .and_then(Self::response_json::<api::KeyListResponse>)
    }

    /// Key create request.
    pub fn key_create(
        &self,
        body: api::KeyCreateRequest,
    ) -> impl Future<Item = api::KeyCreateResponse, Error = ApiError> {
        self.post(api::route::KEY, &body)
            .and_then(Self::response_json::<api::KeyCreateResponse>)
    }

    /// Key read request.
    pub fn key_read(&self, id: Uuid) -> impl Future<Item = api::KeyReadResponse, Error = ApiError> {
        self.get(api::route::key_id(id))
            .and_then(Self::response_json::<api::KeyReadResponse>)
    }

    /// Key update request.
    pub fn key_update(
        &self,
        id: Uuid,
        body: api::KeyUpdateRequest,
    ) -> impl Future<Item = api::KeyReadResponse, Error = ApiError> {
        self.patch(api::route::key_id(id), &body)
            .and_then(Self::response_json::<api::KeyReadResponse>)
    }

    /// Key delete request.
    pub fn key_delete(&self, id: Uuid) -> impl Future<Item = (), Error = ApiError> {
        self.delete(api::route::key_id(id))
            .and_then(Self::response_empty)
    }

    /// Service list request.
    pub fn service_list(
        &self,
        query: api::ServiceListRequest,
    ) -> impl Future<Item = api::ServiceListResponse, Error = ApiError> {
        self.get_query(api::route::SERVICE, query)
            .and_then(Self::response_json::<api::ServiceListResponse>)
    }

    /// Service create request.
    pub fn service_create(
        &self,
        body: api::ServiceCreateRequest,
    ) -> impl Future<Item = api::ServiceReadResponse, Error = ApiError> {
        self.post(api::route::SERVICE, &body)
            .and_then(Self::response_json::<api::ServiceReadResponse>)
    }

    /// Service read request.
    pub fn service_read(
        &self,
        id: Uuid,
    ) -> impl Future<Item = api::ServiceReadResponse, Error = ApiError> {
        self.get(api::route::service_id(id))
            .and_then(Self::response_json::<api::ServiceReadResponse>)
    }

    /// Service update request.
    pub fn service_update(
        &self,
        id: Uuid,
        body: api::ServiceUpdateRequest,
    ) -> impl Future<Item = api::ServiceReadResponse, Error = ApiError> {
        self.patch(api::route::service_id(id), &body)
            .and_then(Self::response_json::<api::ServiceReadResponse>)
    }

    /// Service delete request.
    pub fn service_delete(&self, id: Uuid) -> impl Future<Item = (), Error = ApiError> {
        self.delete(api::route::service_id(id))
            .and_then(Self::response_empty)
    }

    /// User list request.
    pub fn user_list(
        &self,
        query: api::UserListRequest,
    ) -> impl Future<Item = api::UserListResponse, Error = ApiError> {
        self.get_query(api::route::USER, query)
            .and_then(Self::response_json::<api::UserListResponse>)
    }

    /// User create request.
    pub fn user_create(
        &self,
        body: api::UserCreateRequest,
    ) -> impl Future<Item = api::UserCreateResponse, Error = ApiError> {
        self.post(api::route::USER, &body)
            .and_then(Self::response_json::<api::UserCreateResponse>)
    }

    /// User read request.
    pub fn user_read(
        &self,
        id: Uuid,
    ) -> impl Future<Item = api::UserReadResponse, Error = ApiError> {
        self.get(api::route::user_id(id))
            .and_then(Self::response_json::<api::UserReadResponse>)
    }

    /// User update request.
    pub fn user_update(
        &self,
        id: Uuid,
        body: api::UserUpdateRequest,
    ) -> impl Future<Item = api::UserReadResponse, Error = ApiError> {
        self.patch(api::route::user_id(id), &body)
            .and_then(Self::response_json::<api::UserReadResponse>)
    }

    /// User delete request.
    pub fn user_delete(&self, id: Uuid) -> impl Future<Item = (), Error = ApiError> {
        self.delete(api::route::user_id(id))
            .and_then(Self::response_empty)
    }

    fn get<R>(&self, route: R) -> impl Future<Item = String, Error = ApiError>
    where
        R: AsRef<str>,
    {
        let url = ClientOptions::url(self.url(), route.as_ref()).unwrap();
        self.options
            .request_headers(self.client.get(url))
            .send()
            .map_err(Into::into)
            .and_then(|res| res.error_for_status().map_err(Into::into))
            .and_then(|mut res| res.text().map_err(Into::into))
    }

    fn get_query<R, B>(&self, route: R, query: B) -> impl Future<Item = String, Error = ApiError>
    where
        R: AsRef<str>,
        B: Serialize,
    {
        let url = ClientOptions::url_query(self.url(), route.as_ref(), query).unwrap();
        self.options
            .request_headers(self.client.get(url))
            .send()
            .map_err(Into::into)
            .and_then(|res| res.error_for_status().map_err(Into::into))
            .and_then(|mut res| res.text().map_err(Into::into))
    }

    fn post<R, B>(&self, route: R, body: &B) -> impl Future<Item = String, Error = ApiError>
    where
        R: AsRef<str>,
        B: Serialize,
    {
        let url = ClientOptions::url(self.url(), route.as_ref()).unwrap();
        self.options
            .request_headers(self.client.post(url))
            .json(body)
            .send()
            .map_err(Into::into)
            .and_then(|res| res.error_for_status().map_err(Into::into))
            .and_then(|mut res| res.text().map_err(Into::into))
    }

    fn patch<R, B>(&self, route: R, body: &B) -> impl Future<Item = String, Error = ApiError>
    where
        R: AsRef<str>,
        B: Serialize,
    {
        let url = ClientOptions::url(self.url(), route.as_ref()).unwrap();
        self.options
            .request_headers(self.client.patch(url))
            .json(body)
            .send()
            .map_err(Into::into)
            .and_then(|res| res.error_for_status().map_err(Into::into))
            .and_then(|mut res| res.text().map_err(Into::into))
    }

    fn delete<R>(&self, route: R) -> impl Future<Item = String, Error = ApiError>
    where
        R: AsRef<str>,
    {
        let url = ClientOptions::url(self.url(), route.as_ref()).unwrap();
        self.options
            .request_headers(self.client.delete(url))
            .send()
            .map_err(Into::into)
            .and_then(|res| res.error_for_status().map_err(Into::into))
            .and_then(|mut res| res.text().map_err(Into::into))
    }

    /// Deserialise response text into type.
    fn response_json<T: DeserializeOwned>(text: String) -> ApiResult<T> {
        serde_json::from_str::<T>(&text)
            .map_err(DriverError::SerdeJson)
            .map_err(ApiError::BadRequest)
    }

    /// Return response empty.
    pub fn response_empty(_text: String) -> ApiResult<()> {
        Ok(())
    }
}

impl ClientAsync {
    /// Authenticate user using token or key, returns user if successful.
    pub fn authenticate(
        &self,
        key_or_token: Option<String>,
        audit: Option<String>,
    ) -> impl Future<Item = (User, Option<Uuid>), Error = ApiError> {
        match key_or_token {
            Some(key_or_token) => match ClientOptions::authorisation_type(key_or_token)
                .map_err(ApiError::Unauthorised)
            {
                Ok((type_, value)) => {
                    Either::A(Either::A(self.authenticate_inner(type_, value, audit)))
                }
                Err(e) => Either::A(Either::B(future::err(e))),
            },
            None => Either::B(future::err(ApiError::Unauthorised(
                DriverError::AuthenticateKeyOrTokenUndefined,
            ))),
        }
    }

    fn authenticate_inner(
        &self,
        type_: String,
        value: String,
        audit: Option<String>,
    ) -> impl Future<Item = (User, Option<Uuid>), Error = ApiError> {
        match type_.as_ref() {
            "key" => {
                let body = api::AuthKeyRequest::new(value, audit);
                Either::A(Either::A(
                    self.auth_key_verify(body)
                        .map(|res| (res.data.user, res.audit)),
                ))
            }
            "token" => {
                let body = api::AuthTokenRequest::new(value, audit);
                Either::A(Either::B(
                    self.auth_token_verify(body)
                        .map(|res| (res.data.user, res.audit)),
                ))
            }
            _ => Either::B(future::err(ApiError::Unauthorised(
                DriverError::AuthenticateTypeUnknown,
            ))),
        }
    }
}
