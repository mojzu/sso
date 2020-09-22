#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseCsrf {
    pub client_id: String,
    pub created_at: String,
    pub token: String,
    pub ttl: String,
}

impl ResponseCsrf {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseCsrfBuilder<crate::generics::MissingClientId, crate::generics::MissingCreatedAt, crate::generics::MissingToken, crate::generics::MissingTtl> {
        ResponseCsrfBuilder {
            body: Default::default(),
            _client_id: core::marker::PhantomData,
            _created_at: core::marker::PhantomData,
            _token: core::marker::PhantomData,
            _ttl: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> ResponseCsrfPostBuilder {
        ResponseCsrfPostBuilder
    }
}

impl Into<ResponseCsrf> for ResponseCsrfBuilder<crate::generics::ClientIdExists, crate::generics::CreatedAtExists, crate::generics::TokenExists, crate::generics::TtlExists> {
    fn into(self) -> ResponseCsrf {
        self.body
    }
}

/// Builder for [`ResponseCsrf`](./struct.ResponseCsrf.html) object.
#[derive(Debug, Clone)]
pub struct ResponseCsrfBuilder<ClientId, CreatedAt, Token, Ttl> {
    body: self::ResponseCsrf,
    _client_id: core::marker::PhantomData<ClientId>,
    _created_at: core::marker::PhantomData<CreatedAt>,
    _token: core::marker::PhantomData<Token>,
    _ttl: core::marker::PhantomData<Ttl>,
}

impl<ClientId, CreatedAt, Token, Ttl> ResponseCsrfBuilder<ClientId, CreatedAt, Token, Ttl> {
    #[inline]
    pub fn client_id(mut self, value: impl Into<String>) -> ResponseCsrfBuilder<crate::generics::ClientIdExists, CreatedAt, Token, Ttl> {
        self.body.client_id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn created_at(mut self, value: impl Into<String>) -> ResponseCsrfBuilder<ClientId, crate::generics::CreatedAtExists, Token, Ttl> {
        self.body.created_at = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn token(mut self, value: impl Into<String>) -> ResponseCsrfBuilder<ClientId, CreatedAt, crate::generics::TokenExists, Ttl> {
        self.body.token = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn ttl(mut self, value: impl Into<String>) -> ResponseCsrfBuilder<ClientId, CreatedAt, Token, crate::generics::TtlExists> {
        self.body.ttl = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

/// Builder created by [`ResponseCsrf::post`](./struct.ResponseCsrf.html#method.post) method for a `POST` operation associated with `ResponseCsrf`.
#[derive(Debug, Clone)]
pub struct ResponseCsrfPostBuilder;


impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for ResponseCsrfPostBuilder {
    type Output = ResponseCsrf;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/csrf/create".into()
    }
}
