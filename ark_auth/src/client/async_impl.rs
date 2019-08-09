//! # Asynchronous Client
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
use actix::prelude::*;
use futures::{future, Future};
use http::StatusCode;
use http::{header, HeaderMap};
use reqwest::r#async::{Client as ReqwestClient, ClientBuilder, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use url::Url;

/// ## Client Executor Configuration
pub struct ClientExecutorConfiguration {
    user_agent: String,
}

impl ClientExecutorConfiguration {
    pub fn new<T1>(user_agent: T1) -> Self
    where
        T1: Into<String>,
    {
        Self {
            user_agent: user_agent.into(),
        }
    }
}

/// ## Client Executor
/// Reqwest advises not to recreate clients, and clients cannot be sent across threads.
/// The primary use case for asynchronous client is in actix-web routes, which may
/// run across many threads, so to avoid creating a client for each request, the client
/// can be run in an actor thread and used by passing messages.
pub struct ClientExecutor {
    client: ReqwestClient,
}

impl ClientExecutor {
    /// Start client actor with configuration.
    pub fn start(configuration: ClientExecutorConfiguration) -> Addr<Self> {
        Supervisor::start(move |_| {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::USER_AGENT,
                configuration.user_agent.parse().unwrap(),
            );

            let client = ClientBuilder::new()
                .use_rustls_tls()
                .default_headers(headers)
                .build()
                .unwrap();
            Self { client }
        })
    }
}

impl Supervised for ClientExecutor {}

impl Actor for ClientExecutor {
    type Context = Context<Self>;
}

/// ## Asynchronous Client GET Request
#[derive(Debug, Serialize, Deserialize)]
pub struct Get {
    pub url: String,
    pub route: String,
    pub content_type: String,
    pub authorisation: Option<String>,
}

impl Get {
    /// Create new GET text request.
    pub fn text<T1, T2>(url: T1, route: T2) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            url: url.into(),
            route: route.into(),
            content_type: "text/plain".to_owned(),
            authorisation: None,
        }
    }

    /// Create new GET JSON request.
    pub fn json<T1, T2>(url: T1, route: T2) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            url: url.into(),
            route: route.into(),
            content_type: "application/json".to_owned(),
            authorisation: None,
        }
    }

    /// Set authorisation header on GET request.
    pub fn authorisation<T1: Into<String>>(mut self, authorisation: T1) -> Self {
        self.authorisation = Some(authorisation.into());
        self
    }

    /// Build and return Url.
    pub fn url(&self) -> Result<Url, Error> {
        let u = Url::parse(&self.url).map_err(|err| Error::url(&err))?;
        u.join(&self.route).map_err(|err| Error::url(&err))
    }
}

impl Message for Get {
    type Result = Result<String, Error>;
}

impl Handler<Get> for ClientExecutor {
    type Result = ResponseActFuture<Self, String, Error>;

    fn handle(&mut self, msg: Get, _ctx: &mut Context<Self>) -> Self::Result {
        let url = msg.url().unwrap();
        let req = self
            .client
            .get(url)
            .header(header::CONTENT_TYPE, &msg.content_type);
        let req = if let Some(authorisation) = &msg.authorisation {
            req.header(header::AUTHORIZATION, authorisation)
        } else {
            req
        };

        let res = req
            .send()
            .map_err(Into::into)
            .and_then(|res| res.error_for_status().map_err(Into::into))
            .and_then(|mut res| res.text().map_err(Into::into));

        let wrapped = actix::fut::wrap_future(res);
        Box::new(wrapped)
    }
}

//////////////////////////////

/// Asynchronous client handle.
#[derive(Clone)]
pub struct AsyncClient {
    pub options: ClientOptions,
    pub client: ReqwestClient,
}

impl AsyncClient {
    /// Authenticate user using token or key, returns user if successful.
    pub fn authenticate(
        &self,
        key_or_token: Option<String>,
        audit: Option<AuditDataRequest>,
    ) -> impl Future<Item = User, Error = Error> {
        match key_or_token {
            Some(key_or_token) => {
                let (s1, s2) = (self.clone(), self.clone());
                future::Either::A(
                    AsyncClient::split_authorisation(key_or_token)
                        .and_then(move |(type_, value)| match type_.as_ref() {
                            "key" => {
                                let body = AuthKeyBody::new(value, audit);
                                future::Either::A(future::Either::A(
                                    s1.auth_key_verify(body).map(|res| res.data.user_id),
                                ))
                            }
                            "token" => {
                                let body = AuthTokenBody::new(value, audit);
                                future::Either::A(future::Either::B(
                                    s1.auth_token_verify(body).map(|res| res.data.user_id),
                                ))
                            }
                            _ => future::Either::B(future::err(Error::InvalidKeyOrToken)),
                        })
                        .and_then(move |user_id| s2.user_read(&user_id))
                        .map(|res| res.data),
                )
            }
            None => future::Either::B(future::err(Error::InvalidKeyOrToken)),
        }
    }

    /// Authentication local provider login request.
    pub fn auth_local_login(
        &self,
        body: AuthLoginBody,
    ) -> impl Future<Item = AuthLoginResponse, Error = Error> {
        AsyncClient::send_response_json::<AuthLoginResponse>(
            self.post(route::AUTH_LOCAL_LOGIN).json(&body),
        )
    }

    /// Authentication local provider reset password request.
    pub fn auth_local_reset_password(
        &self,
        body: AuthResetPasswordBody,
    ) -> impl Future<Item = (), Error = Error> {
        AsyncClient::send_response_empty(self.post(route::AUTH_LOCAL_RESET_PASSWORD).json(&body))
    }

    /// Authentication local provider reset password confirm request.
    pub fn auth_local_reset_password_confirm(
        &self,
        body: AuthResetPasswordConfirmBody,
    ) -> impl Future<Item = AuthPasswordMetaResponse, Error = Error> {
        AsyncClient::send_response_json::<AuthPasswordMetaResponse>(
            self.post(route::AUTH_LOCAL_RESET_PASSWORD_CONFIRM)
                .json(&body),
        )
    }

    /// Authentication local provider update email request.
    pub fn auth_local_update_email(
        &self,
        body: AuthUpdateEmailBody,
    ) -> impl Future<Item = (), Error = Error> {
        AsyncClient::send_response_empty(self.post(route::AUTH_LOCAL_UPDATE_EMAIL).json(&body))
    }

    /// Authentication local provider update email revoke request.
    pub fn auth_local_update_email_revoke(
        &self,
        body: AuthTokenBody,
    ) -> impl Future<Item = (), Error = Error> {
        AsyncClient::send_response_empty(
            self.post(route::AUTH_LOCAL_UPDATE_EMAIL_REVOKE).json(&body),
        )
    }

    /// Authentication local provider update password request.
    pub fn auth_local_update_password(
        &self,
        body: AuthUpdatePasswordBody,
    ) -> impl Future<Item = (), Error = Error> {
        AsyncClient::send_response_empty(self.post(route::AUTH_LOCAL_UPDATE_PASSWORD).json(&body))
    }

    /// Authentication local provider update password revoke request.
    pub fn auth_local_update_password_revoke(
        &self,
        body: AuthTokenBody,
    ) -> impl Future<Item = (), Error = Error> {
        AsyncClient::send_response_empty(
            self.post(route::AUTH_LOCAL_UPDATE_PASSWORD_REVOKE)
                .json(&body),
        )
    }

    /// Authentication GitHub provider OAuth2 request.
    pub fn auth_github_oauth2_request(
        &self,
    ) -> impl Future<Item = AuthOauth2UrlResponse, Error = Error> {
        AsyncClient::send_response_json::<AuthOauth2UrlResponse>(
            self.post(route::AUTH_GITHUB_OAUTH2),
        )
    }

    /// Authentication Microsoft provider OAuth2 request.
    pub fn auth_microsoft_oauth2_request(
        &self,
    ) -> impl Future<Item = AuthOauth2UrlResponse, Error = Error> {
        AsyncClient::send_response_json::<AuthOauth2UrlResponse>(
            self.post(route::AUTH_MICROSOFT_OAUTH2),
        )
    }

    /// Authentication verify key.
    pub fn auth_key_verify(
        &self,
        body: AuthKeyBody,
    ) -> impl Future<Item = AuthKeyResponse, Error = Error> {
        AsyncClient::send_response_json::<AuthKeyResponse>(
            self.post(route::AUTH_KEY_VERIFY).json(&body),
        )
    }

    /// Authentication revoke key.
    pub fn auth_key_revoke(&self, body: AuthKeyBody) -> impl Future<Item = (), Error = Error> {
        AsyncClient::send_response_empty(self.post(route::AUTH_KEY_REVOKE).json(&body))
    }

    /// Authentication revoke token.
    pub fn auth_token_verify(
        &self,
        body: AuthTokenBody,
    ) -> impl Future<Item = AuthTokenPartialResponse, Error = Error> {
        AsyncClient::send_response_json::<AuthTokenPartialResponse>(
            self.post(route::AUTH_TOKEN_VERIFY).json(&body),
        )
    }

    /// Authentication revoke token.
    pub fn auth_token_refresh(
        &self,
        body: AuthTokenBody,
    ) -> impl Future<Item = AuthTokenResponse, Error = Error> {
        AsyncClient::send_response_json::<AuthTokenResponse>(
            self.post(route::AUTH_TOKEN_REFRESH).json(&body),
        )
    }

    /// Authentication revoke token.
    pub fn auth_token_revoke(&self, body: AuthTokenBody) -> impl Future<Item = (), Error = Error> {
        AsyncClient::send_response_empty(self.post(route::AUTH_TOKEN_REVOKE).json(&body))
    }

    /// Audit list request.
    pub fn audit_list(
        &self,
        query: AuditListQuery,
    ) -> impl Future<Item = AuditListResponse, Error = Error> {
        AsyncClient::send_response_json::<AuditListResponse>(self.get_query(route::AUDIT, query))
    }

    /// Audit create request.
    pub fn audit_create(
        &self,
        body: AuditCreateBody,
    ) -> impl Future<Item = AuditReadResponse, Error = Error> {
        AsyncClient::send_response_json::<AuditReadResponse>(self.post(route::AUDIT).json(&body))
    }

    /// Audit read by ID request.
    pub fn audit_read(&self, id: &str) -> impl Future<Item = AuditReadResponse, Error = Error> {
        let path = route::audit_id(id);
        AsyncClient::send_response_json::<AuditReadResponse>(self.get(&path))
    }

    /// Key list request.
    pub fn key_list(
        &self,
        query: KeyListQuery,
    ) -> impl Future<Item = KeyListResponse, Error = Error> {
        AsyncClient::send_response_json::<KeyListResponse>(self.get_query(route::KEY, query))
    }

    /// Key create request.
    pub fn key_create(
        &self,
        body: KeyCreateBody,
    ) -> impl Future<Item = KeyReadResponse, Error = Error> {
        AsyncClient::send_response_json::<KeyReadResponse>(self.post(route::KEY).json(&body))
    }

    /// Key read request.
    pub fn key_read(&self, id: &str) -> impl Future<Item = KeyReadResponse, Error = Error> {
        let path = route::key_id(id);
        AsyncClient::send_response_json::<KeyReadResponse>(self.get(&path))
    }

    /// Key update request.
    pub fn key_update(
        &self,
        id: &str,
        body: KeyUpdateBody,
    ) -> impl Future<Item = KeyReadResponse, Error = Error> {
        let path = route::key_id(id);
        AsyncClient::send_response_json::<KeyReadResponse>(self.patch(&path).json(&body))
    }

    /// Key delete request.
    pub fn key_delete(&self, id: &str) -> impl Future<Item = (), Error = Error> {
        let path = route::key_id(id);
        AsyncClient::send_response_empty(self.delete(&path))
    }

    /// Service list request.
    pub fn service_list(
        &self,
        query: ServiceListQuery,
    ) -> impl Future<Item = ServiceListResponse, Error = Error> {
        AsyncClient::send_response_json::<ServiceListResponse>(
            self.get_query(route::SERVICE, query),
        )
    }

    /// Service create request.
    pub fn service_create(
        &self,
        body: ServiceCreateBody,
    ) -> impl Future<Item = ServiceReadResponse, Error = Error> {
        AsyncClient::send_response_json::<ServiceReadResponse>(
            self.post(route::SERVICE).json(&body),
        )
    }

    /// Service read request.
    pub fn service_read(&self, id: &str) -> impl Future<Item = ServiceReadResponse, Error = Error> {
        let path = route::service_id(id);
        AsyncClient::send_response_json::<ServiceReadResponse>(self.get(&path))
    }

    /// Service update request.
    pub fn service_update(
        &self,
        id: &str,
        body: ServiceUpdateBody,
    ) -> impl Future<Item = ServiceReadResponse, Error = Error> {
        let path = route::service_id(id);
        AsyncClient::send_response_json::<ServiceReadResponse>(self.patch(&path).json(&body))
    }

    /// Service delete request.
    pub fn service_delete(&self, id: &str) -> impl Future<Item = (), Error = Error> {
        let path = route::service_id(id);
        AsyncClient::send_response_empty(self.delete(&path))
    }

    /// User list request.
    pub fn user_list(
        &self,
        query: UserListQuery,
    ) -> impl Future<Item = UserListResponse, Error = Error> {
        AsyncClient::send_response_json::<UserListResponse>(self.get_query(route::USER, query))
    }

    /// User create request.
    pub fn user_create(
        &self,
        body: UserCreateBody,
    ) -> impl Future<Item = UserCreateResponse, Error = Error> {
        AsyncClient::send_response_json::<UserCreateResponse>(self.post(route::USER).json(&body))
    }

    /// User read request.
    pub fn user_read(&self, id: &str) -> impl Future<Item = UserReadResponse, Error = Error> {
        let path = route::user_id(id);
        AsyncClient::send_response_json::<UserReadResponse>(self.get(&path))
    }

    /// User update request.
    pub fn user_update(
        &self,
        id: &str,
        body: UserUpdateBody,
    ) -> impl Future<Item = UserReadResponse, Error = Error> {
        let path = route::user_id(id);
        AsyncClient::send_response_json::<UserReadResponse>(self.patch(&path).json(&body))
    }

    /// User delete request.
    pub fn user_delete(&self, id: &str) -> impl Future<Item = (), Error = Error> {
        let path = route::user_id(id);
        AsyncClient::send_response_empty(self.delete(&path))
    }

    fn build_client(options: &ClientOptions) -> ReqwestClient {
        let headers = options.default_headers();
        let builder = ClientBuilder::new()
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

    fn send(request: RequestBuilder) -> impl Future<Item = Response, Error = Error> {
        request
            .send()
            .map_err(Into::into)
            .and_then(|response| match response.status() {
                StatusCode::OK => future::ok(response),
                StatusCode::BAD_REQUEST => future::err(Error::Request(RequestError::BadRequest)),
                StatusCode::FORBIDDEN => future::err(Error::Request(RequestError::Forbidden)),
                StatusCode::NOT_FOUND => future::err(Error::Request(RequestError::NotFound)),
                _ => future::err(Error::Response),
            })
    }

    fn send_response_empty(request: RequestBuilder) -> impl Future<Item = (), Error = Error> {
        AsyncClient::send(request).map(|_| ())
    }

    fn send_response_json<T: DeserializeOwned>(
        request: RequestBuilder,
    ) -> impl Future<Item = T, Error = Error> {
        AsyncClient::send(request).and_then(|mut res| res.json::<T>().map_err(Into::into))
    }

    fn split_authorisation(type_value: String) -> future::FutureResult<(String, String), Error> {
        future::result(ClientOptions::split_authorisation(type_value))
    }
}

impl Client for AsyncClient {
    fn new(options: ClientOptions) -> Self {
        let client = AsyncClient::build_client(&options);
        AsyncClient { options, client }
    }
}

// impl actix::Supervised for AsyncClient {}

// impl actix::Actor for AsyncClient {
//     type Context = Context<Self>;
//     // TODO(refactor): Use actor for external HTTP requests instead of recreating clients.
// }

// impl AsyncClient {
//     /// Ping request.
//     pub fn ping(addr: Addr<AsyncClient>) -> impl Future<Item = Value, Error = Error> {
//         addr.send(Get::new(route::PING))
//             .map_err(|_err| Error::Response)
//             .and_then(AsyncClient::response_json)
//     }

//     /// Metrics request.
//     pub fn metrics(addr: Addr<AsyncClient>) -> impl Future<Item = String, Error = Error> {
//         addr.send(Get::new(route::METRICS))
//             .map_err(|_err| Error::Response)
//             .and_then(|res| res)
//     }

//     fn response_json(res: Result<String, Error>) -> Result<Value, Error> {
//         res.map(|x| serde_json::from_str(&x).unwrap())
//     }
// }

// TODO(refactor): Split actor and async client into...
// AsyncClient (actor), move into mod.rs
// async functions or another struct
