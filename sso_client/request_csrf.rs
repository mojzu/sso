#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestCsrf {
    pub token: String,
}

impl RequestCsrf {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestCsrfBuilder<crate::generics::MissingToken> {
        RequestCsrfBuilder {
            body: Default::default(),
            _token: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestCsrfPostBuilder<crate::generics::MissingToken> {
        RequestCsrfPostBuilder {
            body: Default::default(),
            _token: core::marker::PhantomData,
        }
    }
}

impl Into<RequestCsrf> for RequestCsrfBuilder<crate::generics::TokenExists> {
    fn into(self) -> RequestCsrf {
        self.body
    }
}

impl Into<RequestCsrf> for RequestCsrfPostBuilder<crate::generics::TokenExists> {
    fn into(self) -> RequestCsrf {
        self.body
    }
}

/// Builder for [`RequestCsrf`](./struct.RequestCsrf.html) object.
#[derive(Debug, Clone)]
pub struct RequestCsrfBuilder<Token> {
    body: self::RequestCsrf,
    _token: core::marker::PhantomData<Token>,
}

impl<Token> RequestCsrfBuilder<Token> {
    #[inline]
    pub fn token(mut self, value: impl Into<String>) -> RequestCsrfBuilder<crate::generics::TokenExists> {
        self.body.token = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

/// Builder created by [`RequestCsrf::post`](./struct.RequestCsrf.html#method.post) method for a `POST` operation associated with `RequestCsrf`.
#[derive(Debug, Clone)]
pub struct RequestCsrfPostBuilder<Token> {
    body: self::RequestCsrf,
    _token: core::marker::PhantomData<Token>,
}

impl<Token> RequestCsrfPostBuilder<Token> {
    #[inline]
    pub fn token(mut self, value: impl Into<String>) -> RequestCsrfPostBuilder<crate::generics::TokenExists> {
        self.body.token = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestCsrfPostBuilder<crate::generics::TokenExists> {
    type Output = Any<serde_json::Value>;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/csrf/verify".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body)
        .header(http::header::ACCEPT.as_str(), "application/json"))
    }
}
