#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestApiKeyVerify {
    pub key: String,
}

impl RequestApiKeyVerify {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestApiKeyVerifyBuilder<crate::generics::MissingKey> {
        RequestApiKeyVerifyBuilder {
            body: Default::default(),
            _key: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestApiKeyVerifyPostBuilder<crate::generics::MissingKey> {
        RequestApiKeyVerifyPostBuilder {
            body: Default::default(),
            _key: core::marker::PhantomData,
        }
    }
}

impl Into<RequestApiKeyVerify> for RequestApiKeyVerifyBuilder<crate::generics::KeyExists> {
    fn into(self) -> RequestApiKeyVerify {
        self.body
    }
}

impl Into<RequestApiKeyVerify> for RequestApiKeyVerifyPostBuilder<crate::generics::KeyExists> {
    fn into(self) -> RequestApiKeyVerify {
        self.body
    }
}

/// Builder for [`RequestApiKeyVerify`](./struct.RequestApiKeyVerify.html) object.
#[derive(Debug, Clone)]
pub struct RequestApiKeyVerifyBuilder<Key> {
    body: self::RequestApiKeyVerify,
    _key: core::marker::PhantomData<Key>,
}

impl<Key> RequestApiKeyVerifyBuilder<Key> {
    #[inline]
    pub fn key(mut self, value: impl Into<String>) -> RequestApiKeyVerifyBuilder<crate::generics::KeyExists> {
        self.body.key = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

/// Builder created by [`RequestApiKeyVerify::post`](./struct.RequestApiKeyVerify.html#method.post) method for a `POST` operation associated with `RequestApiKeyVerify`.
#[derive(Debug, Clone)]
pub struct RequestApiKeyVerifyPostBuilder<Key> {
    body: self::RequestApiKeyVerify,
    _key: core::marker::PhantomData<Key>,
}

impl<Key> RequestApiKeyVerifyPostBuilder<Key> {
    #[inline]
    pub fn key(mut self, value: impl Into<String>) -> RequestApiKeyVerifyPostBuilder<crate::generics::KeyExists> {
        self.body.key = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestApiKeyVerifyPostBuilder<crate::generics::KeyExists> {
    type Output = crate::response_api_key::ResponseApiKey;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/user/api-key/verify".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body))
    }
}
