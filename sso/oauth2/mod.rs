//! # OAuth2
mod scope;

pub use scope::*;

use std::fmt;
use url::Url;

/// User-agent redirections
/// [RFC](https://tools.ietf.org/html/rfc6749#section-1.7)
pub trait UserRedirectUri {
    /// Returns user-agent redirection URI
    fn user_redirect_uri(&self, uri: Url) -> Url;
}

/// Urlencoded serialization
pub trait SerializeUrlencoded {
    /// Returns serialized urlencoded parameters
    fn serialize_urlencoded(&self) -> String;
}

/// JSON serialization
pub trait SerializeJson {
    /// Returns serialized json parameters
    fn serialize_json(&self) -> String;
}

/// Error response codes
/// [RFC](https://tools.ietf.org/html/rfc6749#section-5.2)
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    /// The request is missing a required parameter
    InvalidRequest,
    /// Client authentication failed
    UnauthorizedClient,
    /// The resource owner or authorization server denied the request
    AccessDenied,
    /// The authorization server does not support obtaining an authorization
    /// code using this method
    UnsupportedResponseType,
    /// The requested scope is invalid, unknown, or malformed
    InvalidScope,
    /// The authorization server encountered an unexpected condition that
    /// prevented it from fulfilling the request
    ServerError,
    /// The authorization server is currently unable to handle the request due
    /// to a temporary overloading or maintenance of the server
    TemporarilyUnavailable,
}

impl ErrorCode {
    /// Returns string representation of error code
    pub fn as_str(&self) -> &str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::UnauthorizedClient => "unauthorized_client",
            Self::AccessDenied => "access_denied",
            Self::UnsupportedResponseType => "unsupported_response_type",
            Self::InvalidScope => "invalid_scope",
            Self::ServerError => "server_error",
            Self::TemporarilyUnavailable => "temporarily_unavailable",
        }
    }
}

/// Error response
/// [RFC](https://tools.ietf.org/html/rfc6749#section-5.2)
#[api_v2_errors(code = 400, code = 401, code = 403, code = 404, code = 500)]
#[derive(Debug, Clone)]
pub struct ErrorResponse {
    error: ErrorCode,
    error_description: String,
}

impl ErrorResponse {
    /// Returns error code
    pub fn error(&self) -> ErrorCode {
        self.error
    }

    /// Returns error description
    pub fn error_description(&self) -> &str {
        &self.error_description
    }

    /// Returns invalid request error response
    pub fn invalid_request<T: Into<String>>(description: T) -> Self {
        Self {
            error: ErrorCode::InvalidRequest,
            error_description: description.into(),
        }
    }

    /// Returns unauthorized client error response
    pub fn unauthorized_client<T: Into<String>>(description: T) -> Self {
        Self {
            error: ErrorCode::UnauthorizedClient,
            error_description: description.into(),
        }
    }

    /// Returns access denied error response
    pub fn access_denied<T: Into<String>>(description: T) -> Self {
        Self {
            error: ErrorCode::AccessDenied,
            error_description: description.into(),
        }
    }

    /// Returns server error response
    pub fn server_error<T: Into<String>>(description: T) -> Self {
        Self {
            error: ErrorCode::ServerError,
            error_description: description.into(),
        }
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl UserRedirectUri for ErrorResponse {
    fn user_redirect_uri(&self, mut uri: Url) -> Url {
        uri.query_pairs_mut()
            .append_pair("error", self.error.as_str())
            .append_pair("error_description", &self.error_description);
        uri
    }
}

/// Result
pub type Result<T> = std::result::Result<T, ErrorResponse>;

/// Authorization code grant request
/// [RFC](https://tools.ietf.org/html/rfc6749#section-4.1.1)
#[derive(Debug, Clone)]
pub struct AuthorizationCodeRequest {
    client_id: String,
    redirect_uri: Url,
    state: String,
    scope: Scope,
}

impl AuthorizationCodeRequest {
    /// Returns new request
    pub fn new<T: Into<Scope>>(client_id: &str, redirect_uri: Url, state: &str, scope: T) -> Self {
        Self {
            client_id: client_id.to_string(),
            redirect_uri,
            state: state.to_string(),
            scope: scope.into(),
        }
    }

    /// Returns client ID
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Returns redirect URI
    pub fn redirect_uri(&self) -> &Url {
        &self.redirect_uri
    }

    /// Returns state
    pub fn state(&self) -> &str {
        &self.state
    }

    /// Returns scope
    pub fn scope(&self) -> &Scope {
        &self.scope
    }
}

impl UserRedirectUri for AuthorizationCodeRequest {
    fn user_redirect_uri(&self, mut uri: Url) -> Url {
        uri.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", self.redirect_uri.as_str())
            .append_pair("state", &self.state)
            .append_pair("scope", &self.scope.to_string());
        uri
    }
}

/// Authorization code response arguments
#[derive(Debug)]
pub struct AuthorizationCodeResponseArgs {
    code: String,
}

impl AuthorizationCodeResponseArgs {
    /// Returns new arguments
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_string(),
        }
    }
}

/// Authorization code response
/// [RFC](https://tools.ietf.org/html/rfc6749#section-4.1.2)
#[derive(Debug)]
pub struct AuthorizationCodeResponse {
    code: String,
    state: String,
}

impl UserRedirectUri for AuthorizationCodeResponse {
    fn user_redirect_uri(&self, mut uri: Url) -> Url {
        uri.query_pairs_mut()
            .append_pair("code", &self.code)
            .append_pair("state", &self.state);
        uri
    }
}

/// Authorization token response arguments
#[derive(Debug)]
pub struct AuthorizationTokenResponseArgs {
    access_token: String,
    expires_in: i64,
    scope: Scope,
}

impl AuthorizationTokenResponseArgs {
    /// Returns new arguments
    pub fn new<T: AsRef<str>>(access_token: &str, expires_in: i64, scope: &[T]) -> Self {
        Self {
            access_token: access_token.to_string(),
            expires_in,
            scope: Scope::from_ref(scope),
        }
    }
}

/// Access token request
/// [RFC](https://tools.ietf.org/html/rfc6749#section-4.1.3)
#[derive(Debug, Clone)]
pub struct AccessTokenRequest {
    code: String,
    redirect_uri: Url,
    client_id: String,
    client_secret: String,
}

impl AccessTokenRequest {
    /// Returns code
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns client secret
    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }
}

impl SerializeUrlencoded for AccessTokenRequest {
    fn serialize_urlencoded(&self) -> String {
        serde_urlencoded::to_string(&[
            ("grant_type", "authorization_code"),
            ("code", &self.code),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("client_id", &self.client_id),
        ])
        .unwrap()
    }
}

impl SerializeJson for AccessTokenRequest {
    fn serialize_json(&self) -> String {
        serde_json::to_string(&json!({
            "grant_type": "authorization_code",
            "code": self.code,
            "redirect_uri": self.redirect_uri.as_str(),
            "client_id": self.client_id,
        }))
        .unwrap()
    }
}

/// Access token response arguments
#[derive(Debug)]
pub struct TokenResponseArgs {
    access_token: String,
    expires_in: i64,
    scope: Scope,
    refresh_token: Option<String>,
}

impl TokenResponseArgs {
    /// Returns new arguments
    pub fn new<S: Into<Scope>>(
        access_token: &str,
        expires_in: i64,
        scope: S,
        refresh_token: Option<&str>,
    ) -> Self {
        Self {
            access_token: access_token.to_string(),
            expires_in,
            scope: scope.into(),
            refresh_token: refresh_token.map(|x| x.to_string()),
        }
    }
}

/// Access token response
/// [RFC](https://tools.ietf.org/html/rfc6749#section-5.1)
#[derive(Debug)]
pub struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    scope: Scope,
    refresh_token: Option<String>,
}

impl SerializeJson for AccessTokenResponse {
    fn serialize_json(&self) -> String {
        serde_json::to_string(&json!({
            "access_token": self.access_token,
            "token_type": self.token_type,
            "expires_in": self.expires_in,
            "scope": self.scope.to_string(),
            "refresh_token": self.refresh_token,
        }))
        .unwrap()
    }
}

/// Refresh token request
/// [RFC](https://tools.ietf.org/html/rfc6749#section-6)
#[derive(Debug, Clone)]
pub struct RefreshTokenRequest {
    refresh_token: String,
    client_secret: String,
}

impl RefreshTokenRequest {
    /// Returns refresh token
    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }

    /// Returns client secret
    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }
}

impl SerializeUrlencoded for RefreshTokenRequest {
    fn serialize_urlencoded(&self) -> String {
        serde_urlencoded::to_string(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", &self.refresh_token),
        ])
        .unwrap()
    }
}

/// Introspection request
/// [RFC](https://tools.ietf.org/html/rfc7662#section-2.1)
#[derive(Debug)]
pub struct IntrospectionRequest {
    token: String,
    client_secret: String,
}

impl IntrospectionRequest {
    /// Returns token
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns client secret
    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }
}

impl SerializeUrlencoded for IntrospectionRequest {
    fn serialize_urlencoded(&self) -> String {
        serde_urlencoded::to_string(&[("token", &self.token)]).unwrap()
    }
}

/// Access token response arguments
#[derive(Debug)]
pub struct IntrospectionResponseArgs {
    scope: Scope,
    username: String,
    sub: String,
}

impl IntrospectionResponseArgs {
    /// Returns new arguments
    pub fn new<S: Into<Scope>>(scope: S, username: &str, sub: &str) -> Self {
        Self {
            scope: scope.into(),
            username: username.to_string(),
            sub: sub.to_string(),
        }
    }
}

/// Introspection response.
/// [RFC](https://tools.ietf.org/html/rfc7662#section-2.2)
#[derive(Debug)]
pub struct IntrospectionResponse {
    scope: Scope,
    client_id: String,
    username: String,
    sub: String,
}

impl SerializeJson for Option<IntrospectionResponse> {
    fn serialize_json(&self) -> String {
        match self {
            Some(res) => serde_json::to_string(&json!({
                "active": true,
                "scope": res.scope.to_string(),
                "client_id": res.client_id,
                "username": res.username,
                "sub": res.sub,
            }))
            .unwrap(),
            None => serde_json::to_string(&json!({
                "active": false,
            }))
            .unwrap(),
        }
    }
}

/// Resource server role trait
/// [RFC](https://tools.ietf.org/html/rfc6749#section-1.1)
pub trait ResourceServerIf {
    /// Returns server introspect URI
    /// [RFC](https://tools.ietf.org/html/rfc7662#section-2)
    fn server_introspect_uri(&self) -> Url;

    /// Returns client public ID
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-2.2)
    fn client_id(&self) -> String;

    /// Returns client secret key
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-2.3)
    fn client_secret(&self) -> String;

    /// Introspection request
    /// [RFC](https://tools.ietf.org/html/rfc7662#section-2.1)
    fn oauth2_introspection_request(&self, token: &str) -> (IntrospectionRequest, Url) {
        let req = IntrospectionRequest {
            token: token.to_string(),
            client_secret: self.client_secret(),
        };
        (req, self.server_introspect_uri())
    }
}

/// Client role trait
/// [RFC](https://tools.ietf.org/html/rfc6749#section-1.1)
pub trait ClientIf {
    /// Returns server authorize URI
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-3.1)
    fn server_authorize_uri(&self) -> Url;

    /// Returns server token URI
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-3.2)
    fn server_token_uri(&self) -> Url;

    /// Returns client public ID
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-2.2)
    fn client_id(&self) -> String;

    /// Returns client secret key
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-2.3)
    fn client_secret(&self) -> String;

    /// Returns registered client redirect URI
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-3.1.2)
    fn redirect_uri(&self) -> Url;

    /// Authorization code grant request
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-4.1.1)
    fn oauth2_authorization_code_request<T: Into<Scope>>(
        &self,
        state: &str,
        scope: T,
    ) -> (AuthorizationCodeRequest, Url) {
        let req =
            AuthorizationCodeRequest::new(&self.client_id(), self.redirect_uri(), state, scope);
        let uri = req.user_redirect_uri(self.server_authorize_uri());
        (req, uri)
    }

    /// Authorization code grant access token request
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-4.1.3)
    fn oauth2_access_token_request(
        &self,
        authorization_response: AuthorizationCodeResponse,
    ) -> (AccessTokenRequest, Url) {
        let req = AccessTokenRequest {
            code: authorization_response.code,
            redirect_uri: self.redirect_uri(),
            client_id: self.client_id(),
            client_secret: self.client_secret(),
        };
        (req, self.server_token_uri())
    }

    /// Refresh access token request
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-6)
    fn oauth2_refresh_token_request(&self, refresh_token: &str) -> (RefreshTokenRequest, Url) {
        let req = RefreshTokenRequest {
            refresh_token: refresh_token.to_string(),
            client_secret: self.client_secret(),
        };
        (req, self.server_token_uri())
    }
}

/// Parsed token request for code grant (access token) or refresh token
#[derive(Debug, Clone)]
pub enum TokenParseRequest {
    /// Access token request
    Access(AccessTokenRequest),
    /// Refresh token request
    Refresh(RefreshTokenRequest),
}

impl TokenParseRequest {
    /// Returns client secret
    pub fn client_secret(&self) -> &str {
        match self {
            Self::Access(x) => &x.client_secret,
            Self::Refresh(x) => &x.client_secret,
        }
    }
}

/// Authorization role trait
/// [RFC](https://tools.ietf.org/html/rfc6749#section-1.1)
pub trait AuthorizationServerIf {
    /// Parse parameters to authorize endpoint into code or token request
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-4.1.2.1)
    fn oauth2_authorize_parse_request(
        &self,
        response_type: Option<&str>,
        client_id: Option<&str>,
        redirect_uri: Option<&str>,
        state: Option<&str>,
        scope: Option<&str>,
    ) -> Result<AuthorizationCodeRequest> {
        let response_type = if let Some(response_type) = response_type {
            response_type
        } else {
            return Err(ErrorResponse::invalid_request("response_type is required"));
        };
        let client_id = if let Some(client_id) = client_id {
            client_id
        } else {
            return Err(ErrorResponse::invalid_request("client_id is required"));
        };
        let redirect_uri = if let Some(redirect_uri) = redirect_uri {
            match Url::parse(redirect_uri) {
                Ok(redirect_uri) => redirect_uri,
                Err(_e) => {
                    return Err(ErrorResponse::invalid_request("redirect_uri is invalid"));
                }
            }
        } else {
            return Err(ErrorResponse::invalid_request("redirect_uri is required"));
        };
        let state = if let Some(state) = state {
            state
        } else {
            return Err(ErrorResponse::invalid_request("state is required"));
        };
        let scope = if let Some(scope) = scope {
            Scope::from_string(scope)
        } else {
            Scope::default()
        };

        match response_type {
            "code" => Ok(AuthorizationCodeRequest::new(
                client_id,
                redirect_uri,
                state,
                scope,
            )),
            _ => Err(ErrorResponse::invalid_request("response_type is invalid")),
        }
    }

    /// Parse parameters to token endpoint into access or refresh token request
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-4.1.2.1)
    fn oauth2_token_parse_request(
        &self,
        grant_type: Option<&str>,
        code: Option<&str>,
        redirect_uri: Option<&str>,
        client_id: Option<&str>,
        refresh_token: Option<&str>,
        client_secret: Option<&str>,
    ) -> Result<TokenParseRequest> {
        let grant_type = if let Some(grant_type) = grant_type {
            grant_type
        } else {
            return Err(ErrorResponse::invalid_request("grant_type is required"));
        };
        let client_secret = if let Some(client_secret) = client_secret {
            client_secret.to_string()
        } else {
            return Err(ErrorResponse::invalid_request("client_secret is required"));
        };

        match grant_type {
            "authorization_code" => {
                let code = if let Some(code) = code {
                    code.to_string()
                } else {
                    return Err(ErrorResponse::invalid_request("code is required"));
                };
                let redirect_uri = if let Some(redirect_uri) = redirect_uri {
                    match Url::parse(redirect_uri) {
                        Ok(redirect_uri) => redirect_uri,
                        Err(_e) => {
                            return Err(ErrorResponse::invalid_request("redirect_uri is invalid"));
                        }
                    }
                } else {
                    return Err(ErrorResponse::invalid_request("redirect_uri is required"));
                };
                let client_id = if let Some(client_id) = client_id {
                    client_id.to_string()
                } else {
                    return Err(ErrorResponse::invalid_request("client_id is required"));
                };

                Ok(TokenParseRequest::Access(AccessTokenRequest {
                    code,
                    redirect_uri,
                    client_id,
                    client_secret,
                }))
            }
            "refresh_token" => {
                let refresh_token = if let Some(refresh_token) = refresh_token {
                    refresh_token.to_string()
                } else {
                    return Err(ErrorResponse::invalid_request("refresh_token is required"));
                };

                Ok(TokenParseRequest::Refresh(RefreshTokenRequest {
                    refresh_token,
                    client_secret,
                }))
            }
            _ => Err(ErrorResponse::invalid_request("grant_type is invalid")),
        }
    }

    /// Parse parameters to introspect endpoint into token request
    /// [RFC](https://tools.ietf.org/html/rfc7662#section-2.3)
    fn oauth2_introspect_parse_request(
        &self,
        token: Option<&str>,
        client_secret: Option<&str>,
    ) -> Result<IntrospectionRequest> {
        let token = if let Some(token) = token {
            token.to_string()
        } else {
            return Err(ErrorResponse::invalid_request("token is required"));
        };
        let client_secret = if let Some(client_secret) = client_secret {
            client_secret.to_string()
        } else {
            return Err(ErrorResponse::invalid_request("client_secret is required"));
        };
        Ok(IntrospectionRequest {
            token,
            client_secret,
        })
    }

    /// Authorization code grant response
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-4.1.2)
    fn oauth2_authorization_code_response<C: ClientIf>(
        &self,
        client: &C,
        request: AuthorizationCodeRequest,
        args: AuthorizationCodeResponseArgs,
    ) -> Result<(AuthorizationCodeResponse, Url)> {
        let res = AuthorizationCodeResponse {
            code: args.code,
            state: request.state,
        };
        let uri = res.user_redirect_uri(client.redirect_uri());
        Ok((res, uri))
    }

    /// Authorization code grant access token response
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-4.1.4)
    fn oauth2_access_token_response<C: ClientIf>(
        &self,
        _client: &C,
        _request: AccessTokenRequest,
        args: TokenResponseArgs,
    ) -> AccessTokenResponse {
        AccessTokenResponse {
            access_token: args.access_token.to_string(),
            token_type: "bearer".to_string(),
            expires_in: args.expires_in,
            scope: args.scope,
            refresh_token: args.refresh_token,
        }
    }

    /// Refresh access token response
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-6)
    fn oauth2_refresh_token_response<C: ClientIf>(
        &self,
        _client: &C,
        _request: RefreshTokenRequest,
        args: TokenResponseArgs,
    ) -> AccessTokenResponse {
        AccessTokenResponse {
            access_token: args.access_token.to_string(),
            token_type: "bearer".to_string(),
            expires_in: args.expires_in,
            scope: args.scope,
            refresh_token: args.refresh_token,
        }
    }

    /// Introspection response
    /// [RFC](https://tools.ietf.org/html/rfc7662#section-2.2)
    fn oauth2_introspection_response<C: ClientIf>(
        &self,
        client: &C,
        _request: IntrospectionRequest,
        args: IntrospectionResponseArgs,
    ) -> IntrospectionResponse {
        IntrospectionResponse {
            scope: args.scope,
            client_id: client.client_id(),
            username: args.username,
            sub: args.sub,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct SsoServer {}
    impl AuthorizationServerIf for SsoServer {}

    #[derive(Debug)]
    struct ServiceClient {
        server_authorize_uri: Url,
        server_token_uri: Url,
        server_introspect_uri: Url,
        client_id: String,
        client_secret: String,
        redirect_uri: Url,
    }
    impl ResourceServerIf for ServiceClient {
        fn server_introspect_uri(&self) -> Url {
            self.server_introspect_uri.clone()
        }
        fn client_id(&self) -> String {
            self.client_id.clone()
        }
        fn client_secret(&self) -> String {
            self.client_secret.clone()
        }
    }
    impl ClientIf for ServiceClient {
        fn server_authorize_uri(&self) -> Url {
            self.server_authorize_uri.clone()
        }
        fn server_token_uri(&self) -> Url {
            self.server_token_uri.clone()
        }
        fn client_id(&self) -> String {
            self.client_id.clone()
        }
        fn client_secret(&self) -> String {
            self.client_secret.clone()
        }
        fn redirect_uri(&self) -> Url {
            self.redirect_uri.clone()
        }
    }

    #[test]
    fn test_authorization_code_grant() {
        let server_authorize_uri = Url::parse("http://localhost:1000/authorize").unwrap();
        let server_token_uri = Url::parse("http://localhost:1000/token").unwrap();
        let server_introspect_uri = Url::parse("http://localhost:1000/introspect").unwrap();
        let redirect_uri = Url::parse("http://localhost:7420").unwrap();

        let server = SsoServer {};
        let client: ServiceClient = ServiceClient {
            server_authorize_uri,
            server_token_uri,
            server_introspect_uri,
            client_id: "client-id-xyz".to_string(),
            client_secret: "client-secret-xyz".to_string(),
            redirect_uri,
        };

        // Client redirects resource owner to authorization server via constructed URI

        let (auth_req, auth_uri) =
            client.oauth2_authorization_code_request("state-xyz", vec!["scope-1", "scope-2"]);
        assert_eq!(
            auth_uri.as_str(),
            "http://localhost:1000/authorize?response_type=code&client_id=client-id-xyz&redirect_uri=http%3A%2F%2Flocalhost%3A7420%2F&state=state-xyz&scope=scope-1+scope-2"
        );

        // Authorization server authenticates resource owner, if successful redirects resource owner to client via constructed URI

        let (auth_res, auth_uri) = server
            .oauth2_authorization_code_response(
                &client,
                auth_req,
                AuthorizationCodeResponseArgs::new("auth-code-xyz"),
            )
            .unwrap();
        assert_eq!(
            auth_uri.as_str(),
            "http://localhost:7420/?code=auth-code-xyz&state=state-xyz"
        );

        // Client receives code from parameters, makes request to authorization server for access token

        let (access_token_req, _) = client.oauth2_access_token_request(auth_res);
        assert_eq!(
            access_token_req.serialize_urlencoded(),
            "grant_type=authorization_code&code=auth-code-xyz&redirect_uri=http%3A%2F%2Flocalhost%3A7420%2F&client_id=client-id-xyz"
        );

        // Authorization server authenticates client and code, if successful returns access token response

        let access_token_res = server.oauth2_access_token_response(
            &client,
            access_token_req,
            TokenResponseArgs::new(
                "access-token-xyz",
                3600,
                vec!["scope-1", "scope-2"],
                Some("refresh-token-xyz"),
            ),
        );
        assert_eq!(
            access_token_res.serialize_json(),
            r#"{"access_token":"access-token-xyz","expires_in":3600,"refresh_token":"refresh-token-xyz","scope":"scope-1 scope-2","token_type":"bearer"}"#
        );

        // Resource server makes request to authorization server to introspect token

        let (introspect_req, _) =
            client.oauth2_introspection_request(&access_token_res.access_token);
        assert_eq!(
            introspect_req.serialize_urlencoded(),
            "token=access-token-xyz"
        );

        // Authorization server authenticates resource server, if successful returns token information

        let introspect_res = server.oauth2_introspection_response(
            &client,
            introspect_req,
            IntrospectionResponseArgs::new(
                vec!["scope-1", "scope-2"],
                "username-xyz",
                "user-id-xyz",
            ),
        );

        assert_eq!(
            Some(introspect_res).serialize_json(),
            r#"{"active":true,"client_id":"client-id-xyz","scope":"scope-1 scope-2","sub":"user-id-xyz","username":"username-xyz"}"#
        );

        // If authorization server returned refresh token, client makes request to authorization server to refresh token

        if let Some(refresh_token) = access_token_res.refresh_token {
            let (refresh_token_req, _) = client.oauth2_refresh_token_request(&refresh_token);
            assert_eq!(
                refresh_token_req.serialize_urlencoded(),
                "grant_type=refresh_token&refresh_token=refresh-token-xyz"
            );

            // Authorization server authenticates client and refresh token, if successful returns access token response

            let refresh_token_res = server.oauth2_refresh_token_response(
                &client,
                refresh_token_req,
                TokenResponseArgs::new(
                    "access-token-xyz",
                    3600,
                    vec!["scope-1", "scope-2"],
                    Some("refresh-token-xyz"),
                ),
            );
            assert_eq!(
                refresh_token_res.serialize_json(),
                r#"{"access_token":"access-token-xyz","expires_in":3600,"refresh_token":"refresh-token-xyz","scope":"scope-1 scope-2","token_type":"bearer"}"#
            );
        }
    }
}
