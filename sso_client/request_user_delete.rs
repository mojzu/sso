#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestUserDelete {
    pub id: String,
}

impl RequestUserDelete {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestUserDeleteBuilder<crate::generics::MissingId> {
        RequestUserDeleteBuilder {
            body: Default::default(),
            _id: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestUserDeletePostBuilder<crate::generics::MissingId> {
        RequestUserDeletePostBuilder {
            body: Default::default(),
            _id: core::marker::PhantomData,
        }
    }
}

impl Into<RequestUserDelete> for RequestUserDeleteBuilder<crate::generics::IdExists> {
    fn into(self) -> RequestUserDelete {
        self.body
    }
}

impl Into<RequestUserDelete> for RequestUserDeletePostBuilder<crate::generics::IdExists> {
    fn into(self) -> RequestUserDelete {
        self.body
    }
}

/// Builder for [`RequestUserDelete`](./struct.RequestUserDelete.html) object.
#[derive(Debug, Clone)]
pub struct RequestUserDeleteBuilder<Id> {
    body: self::RequestUserDelete,
    _id: core::marker::PhantomData<Id>,
}

impl<Id> RequestUserDeleteBuilder<Id> {
    #[inline]
    pub fn id(mut self, value: impl Into<String>) -> RequestUserDeleteBuilder<crate::generics::IdExists> {
        self.body.id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

/// Builder created by [`RequestUserDelete::post`](./struct.RequestUserDelete.html#method.post) method for a `POST` operation associated with `RequestUserDelete`.
#[derive(Debug, Clone)]
pub struct RequestUserDeletePostBuilder<Id> {
    body: self::RequestUserDelete,
    _id: core::marker::PhantomData<Id>,
}

impl<Id> RequestUserDeletePostBuilder<Id> {
    #[inline]
    pub fn id(mut self, value: impl Into<String>) -> RequestUserDeletePostBuilder<crate::generics::IdExists> {
        self.body.id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestUserDeletePostBuilder<crate::generics::IdExists> {
    type Output = Any<serde_json::Value>;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/user/delete".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body)
        .header(http::header::ACCEPT.as_str(), "application/json"))
    }
}
