#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestApiKeyDelete {
    pub id: String,
}

impl RequestApiKeyDelete {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestApiKeyDeleteBuilder<crate::generics::MissingId> {
        RequestApiKeyDeleteBuilder {
            body: Default::default(),
            _id: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestApiKeyDeletePostBuilder<crate::generics::MissingId> {
        RequestApiKeyDeletePostBuilder {
            body: Default::default(),
            _id: core::marker::PhantomData,
        }
    }
}

impl Into<RequestApiKeyDelete> for RequestApiKeyDeleteBuilder<crate::generics::IdExists> {
    fn into(self) -> RequestApiKeyDelete {
        self.body
    }
}

impl Into<RequestApiKeyDelete> for RequestApiKeyDeletePostBuilder<crate::generics::IdExists> {
    fn into(self) -> RequestApiKeyDelete {
        self.body
    }
}

/// Builder for [`RequestApiKeyDelete`](./struct.RequestApiKeyDelete.html) object.
#[derive(Debug, Clone)]
pub struct RequestApiKeyDeleteBuilder<Id> {
    body: self::RequestApiKeyDelete,
    _id: core::marker::PhantomData<Id>,
}

impl<Id> RequestApiKeyDeleteBuilder<Id> {
    #[inline]
    pub fn id(mut self, value: impl Into<String>) -> RequestApiKeyDeleteBuilder<crate::generics::IdExists> {
        self.body.id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

/// Builder created by [`RequestApiKeyDelete::post`](./struct.RequestApiKeyDelete.html#method.post) method for a `POST` operation associated with `RequestApiKeyDelete`.
#[derive(Debug, Clone)]
pub struct RequestApiKeyDeletePostBuilder<Id> {
    body: self::RequestApiKeyDelete,
    _id: core::marker::PhantomData<Id>,
}

impl<Id> RequestApiKeyDeletePostBuilder<Id> {
    #[inline]
    pub fn id(mut self, value: impl Into<String>) -> RequestApiKeyDeletePostBuilder<crate::generics::IdExists> {
        self.body.id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestApiKeyDeletePostBuilder<crate::generics::IdExists> {
    type Output = Any<serde_json::Value>;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/user/api-key/delete".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body)
        .header(http::header::ACCEPT.as_str(), "application/json"))
    }
}
