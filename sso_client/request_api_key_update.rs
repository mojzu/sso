#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestApiKeyUpdate {
    pub enable: Option<bool>,
    pub id: String,
    pub name: Option<String>,
}

impl RequestApiKeyUpdate {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestApiKeyUpdateBuilder<crate::generics::MissingId> {
        RequestApiKeyUpdateBuilder {
            body: Default::default(),
            _id: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestApiKeyUpdatePostBuilder<crate::generics::MissingId> {
        RequestApiKeyUpdatePostBuilder {
            body: Default::default(),
            _id: core::marker::PhantomData,
        }
    }
}

impl Into<RequestApiKeyUpdate> for RequestApiKeyUpdateBuilder<crate::generics::IdExists> {
    fn into(self) -> RequestApiKeyUpdate {
        self.body
    }
}

impl Into<RequestApiKeyUpdate> for RequestApiKeyUpdatePostBuilder<crate::generics::IdExists> {
    fn into(self) -> RequestApiKeyUpdate {
        self.body
    }
}

/// Builder for [`RequestApiKeyUpdate`](./struct.RequestApiKeyUpdate.html) object.
#[derive(Debug, Clone)]
pub struct RequestApiKeyUpdateBuilder<Id> {
    body: self::RequestApiKeyUpdate,
    _id: core::marker::PhantomData<Id>,
}

impl<Id> RequestApiKeyUpdateBuilder<Id> {
    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> Self {
        self.body.enable = Some(value.into());
        self
    }

    #[inline]
    pub fn id(mut self, value: impl Into<String>) -> RequestApiKeyUpdateBuilder<crate::generics::IdExists> {
        self.body.id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.body.name = Some(value.into());
        self
    }
}

/// Builder created by [`RequestApiKeyUpdate::post`](./struct.RequestApiKeyUpdate.html#method.post) method for a `POST` operation associated with `RequestApiKeyUpdate`.
#[derive(Debug, Clone)]
pub struct RequestApiKeyUpdatePostBuilder<Id> {
    body: self::RequestApiKeyUpdate,
    _id: core::marker::PhantomData<Id>,
}

impl<Id> RequestApiKeyUpdatePostBuilder<Id> {
    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> Self {
        self.body.enable = Some(value.into());
        self
    }

    #[inline]
    pub fn id(mut self, value: impl Into<String>) -> RequestApiKeyUpdatePostBuilder<crate::generics::IdExists> {
        self.body.id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.body.name = Some(value.into());
        self
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestApiKeyUpdatePostBuilder<crate::generics::IdExists> {
    type Output = crate::response_api_key::ResponseApiKey;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/user/api-key/update".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body))
    }
}
