#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestOauth2Token {
    pub code: Option<String>,
    pub grant_type: String,
    pub redirect_uri: Option<String>,
    pub refresh_token: Option<String>,
}

impl RequestOauth2Token {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestOauth2TokenBuilder<crate::generics::MissingGrantType> {
        RequestOauth2TokenBuilder {
            body: Default::default(),
            _grant_type: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestOauth2TokenPostBuilder<crate::generics::MissingGrantType> {
        RequestOauth2TokenPostBuilder {
            body: Default::default(),
            _grant_type: core::marker::PhantomData,
        }
    }
}

impl Into<RequestOauth2Token> for RequestOauth2TokenBuilder<crate::generics::GrantTypeExists> {
    fn into(self) -> RequestOauth2Token {
        self.body
    }
}

impl Into<RequestOauth2Token> for RequestOauth2TokenPostBuilder<crate::generics::GrantTypeExists> {
    fn into(self) -> RequestOauth2Token {
        self.body
    }
}

/// Builder for [`RequestOauth2Token`](./struct.RequestOauth2Token.html) object.
#[derive(Debug, Clone)]
pub struct RequestOauth2TokenBuilder<GrantType> {
    body: self::RequestOauth2Token,
    _grant_type: core::marker::PhantomData<GrantType>,
}

impl<GrantType> RequestOauth2TokenBuilder<GrantType> {
    #[inline]
    pub fn code(mut self, value: impl Into<String>) -> Self {
        self.body.code = Some(value.into());
        self
    }

    #[inline]
    pub fn grant_type(mut self, value: impl Into<String>) -> RequestOauth2TokenBuilder<crate::generics::GrantTypeExists> {
        self.body.grant_type = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn redirect_uri(mut self, value: impl Into<String>) -> Self {
        self.body.redirect_uri = Some(value.into());
        self
    }

    #[inline]
    pub fn refresh_token(mut self, value: impl Into<String>) -> Self {
        self.body.refresh_token = Some(value.into());
        self
    }
}

/// Builder created by [`RequestOauth2Token::post`](./struct.RequestOauth2Token.html#method.post) method for a `POST` operation associated with `RequestOauth2Token`.
#[derive(Debug, Clone)]
pub struct RequestOauth2TokenPostBuilder<GrantType> {
    body: self::RequestOauth2Token,
    _grant_type: core::marker::PhantomData<GrantType>,
}

impl<GrantType> RequestOauth2TokenPostBuilder<GrantType> {
    #[inline]
    pub fn code(mut self, value: impl Into<String>) -> Self {
        self.body.code = Some(value.into());
        self
    }

    #[inline]
    pub fn grant_type(mut self, value: impl Into<String>) -> RequestOauth2TokenPostBuilder<crate::generics::GrantTypeExists> {
        self.body.grant_type = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn redirect_uri(mut self, value: impl Into<String>) -> Self {
        self.body.redirect_uri = Some(value.into());
        self
    }

    #[inline]
    pub fn refresh_token(mut self, value: impl Into<String>) -> Self {
        self.body.refresh_token = Some(value.into());
        self
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestOauth2TokenPostBuilder<crate::generics::GrantTypeExists> {
    type Output = serde_json::Value;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/oauth2/token".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body)
        .header(http::header::ACCEPT.as_str(), "application/json"))
    }
}
