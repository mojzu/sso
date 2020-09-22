#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestOauth2Introspect {
    pub token: String,
}

impl RequestOauth2Introspect {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestOauth2IntrospectBuilder<crate::generics::MissingToken> {
        RequestOauth2IntrospectBuilder {
            body: Default::default(),
            _token: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestOauth2IntrospectPostBuilder<crate::generics::MissingToken> {
        RequestOauth2IntrospectPostBuilder {
            body: Default::default(),
            _token: core::marker::PhantomData,
        }
    }
}

impl Into<RequestOauth2Introspect> for RequestOauth2IntrospectBuilder<crate::generics::TokenExists> {
    fn into(self) -> RequestOauth2Introspect {
        self.body
    }
}

impl Into<RequestOauth2Introspect> for RequestOauth2IntrospectPostBuilder<crate::generics::TokenExists> {
    fn into(self) -> RequestOauth2Introspect {
        self.body
    }
}

/// Builder for [`RequestOauth2Introspect`](./struct.RequestOauth2Introspect.html) object.
#[derive(Debug, Clone)]
pub struct RequestOauth2IntrospectBuilder<Token> {
    body: self::RequestOauth2Introspect,
    _token: core::marker::PhantomData<Token>,
}

impl<Token> RequestOauth2IntrospectBuilder<Token> {
    #[inline]
    pub fn token(mut self, value: impl Into<String>) -> RequestOauth2IntrospectBuilder<crate::generics::TokenExists> {
        self.body.token = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

/// Builder created by [`RequestOauth2Introspect::post`](./struct.RequestOauth2Introspect.html#method.post) method for a `POST` operation associated with `RequestOauth2Introspect`.
#[derive(Debug, Clone)]
pub struct RequestOauth2IntrospectPostBuilder<Token> {
    body: self::RequestOauth2Introspect,
    _token: core::marker::PhantomData<Token>,
}

impl<Token> RequestOauth2IntrospectPostBuilder<Token> {
    #[inline]
    pub fn token(mut self, value: impl Into<String>) -> RequestOauth2IntrospectPostBuilder<crate::generics::TokenExists> {
        self.body.token = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestOauth2IntrospectPostBuilder<crate::generics::TokenExists> {
    type Output = serde_json::Value;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/oauth2/introspect".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body)
        .header(http::header::ACCEPT.as_str(), "application/json"))
    }
}
