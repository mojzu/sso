#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestUserAccessRead {
    pub user_id: String,
}

impl RequestUserAccessRead {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestUserAccessReadBuilder<crate::generics::MissingUserId> {
        RequestUserAccessReadBuilder {
            body: Default::default(),
            _user_id: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestUserAccessReadPostBuilder<crate::generics::MissingUserId> {
        RequestUserAccessReadPostBuilder {
            body: Default::default(),
            _user_id: core::marker::PhantomData,
        }
    }
}

impl Into<RequestUserAccessRead> for RequestUserAccessReadBuilder<crate::generics::UserIdExists> {
    fn into(self) -> RequestUserAccessRead {
        self.body
    }
}

impl Into<RequestUserAccessRead> for RequestUserAccessReadPostBuilder<crate::generics::UserIdExists> {
    fn into(self) -> RequestUserAccessRead {
        self.body
    }
}

/// Builder for [`RequestUserAccessRead`](./struct.RequestUserAccessRead.html) object.
#[derive(Debug, Clone)]
pub struct RequestUserAccessReadBuilder<UserId> {
    body: self::RequestUserAccessRead,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<UserId> RequestUserAccessReadBuilder<UserId> {
    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> RequestUserAccessReadBuilder<crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

/// Builder created by [`RequestUserAccessRead::post`](./struct.RequestUserAccessRead.html#method.post) method for a `POST` operation associated with `RequestUserAccessRead`.
#[derive(Debug, Clone)]
pub struct RequestUserAccessReadPostBuilder<UserId> {
    body: self::RequestUserAccessRead,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<UserId> RequestUserAccessReadPostBuilder<UserId> {
    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> RequestUserAccessReadPostBuilder<crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestUserAccessReadPostBuilder<crate::generics::UserIdExists> {
    type Output = crate::response_access::ResponseAccess;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/user/access/read".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body))
    }
}
