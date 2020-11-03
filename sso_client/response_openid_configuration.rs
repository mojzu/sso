#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseOpenidConfiguration {
    pub authorization_endpoint: String,
    pub issuer: String,
    pub token_endpoint: String,
    pub token_endpoint_auth_methods_supported: Vec<String>,
}

impl ResponseOpenidConfiguration {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseOpenidConfigurationBuilder<crate::generics::MissingAuthorizationEndpoint, crate::generics::MissingIssuer, crate::generics::MissingTokenEndpoint, crate::generics::MissingTokenEndpointAuthMethodsSupported> {
        ResponseOpenidConfigurationBuilder {
            body: Default::default(),
            _authorization_endpoint: core::marker::PhantomData,
            _issuer: core::marker::PhantomData,
            _token_endpoint: core::marker::PhantomData,
            _token_endpoint_auth_methods_supported: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn get() -> ResponseOpenidConfigurationGetBuilder {
        ResponseOpenidConfigurationGetBuilder
    }
}

impl Into<ResponseOpenidConfiguration> for ResponseOpenidConfigurationBuilder<crate::generics::AuthorizationEndpointExists, crate::generics::IssuerExists, crate::generics::TokenEndpointExists, crate::generics::TokenEndpointAuthMethodsSupportedExists> {
    fn into(self) -> ResponseOpenidConfiguration {
        self.body
    }
}

/// Builder for [`ResponseOpenidConfiguration`](./struct.ResponseOpenidConfiguration.html) object.
#[derive(Debug, Clone)]
pub struct ResponseOpenidConfigurationBuilder<AuthorizationEndpoint, Issuer, TokenEndpoint, TokenEndpointAuthMethodsSupported> {
    body: self::ResponseOpenidConfiguration,
    _authorization_endpoint: core::marker::PhantomData<AuthorizationEndpoint>,
    _issuer: core::marker::PhantomData<Issuer>,
    _token_endpoint: core::marker::PhantomData<TokenEndpoint>,
    _token_endpoint_auth_methods_supported: core::marker::PhantomData<TokenEndpointAuthMethodsSupported>,
}

impl<AuthorizationEndpoint, Issuer, TokenEndpoint, TokenEndpointAuthMethodsSupported> ResponseOpenidConfigurationBuilder<AuthorizationEndpoint, Issuer, TokenEndpoint, TokenEndpointAuthMethodsSupported> {
    #[inline]
    pub fn authorization_endpoint(mut self, value: impl Into<String>) -> ResponseOpenidConfigurationBuilder<crate::generics::AuthorizationEndpointExists, Issuer, TokenEndpoint, TokenEndpointAuthMethodsSupported> {
        self.body.authorization_endpoint = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn issuer(mut self, value: impl Into<String>) -> ResponseOpenidConfigurationBuilder<AuthorizationEndpoint, crate::generics::IssuerExists, TokenEndpoint, TokenEndpointAuthMethodsSupported> {
        self.body.issuer = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn token_endpoint(mut self, value: impl Into<String>) -> ResponseOpenidConfigurationBuilder<AuthorizationEndpoint, Issuer, crate::generics::TokenEndpointExists, TokenEndpointAuthMethodsSupported> {
        self.body.token_endpoint = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn token_endpoint_auth_methods_supported(mut self, value: impl Iterator<Item = impl Into<String>>) -> ResponseOpenidConfigurationBuilder<AuthorizationEndpoint, Issuer, TokenEndpoint, crate::generics::TokenEndpointAuthMethodsSupportedExists> {
        self.body.token_endpoint_auth_methods_supported = value.map(|value| value.into()).collect::<Vec<_>>().into();
        unsafe { std::mem::transmute(self) }
    }
}

/// Builder created by [`ResponseOpenidConfiguration::get`](./struct.ResponseOpenidConfiguration.html#method.get) method for a `GET` operation associated with `ResponseOpenidConfiguration`.
#[derive(Debug, Clone)]
pub struct ResponseOpenidConfigurationGetBuilder;


impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for ResponseOpenidConfigurationGetBuilder {
    type Output = ResponseOpenidConfiguration;

    const METHOD: http::Method = http::Method::GET;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/.well-known/openid-configuration".into()
    }
}
