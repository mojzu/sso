#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestAccessDelete {
    pub user_id: String,
}

impl RequestAccessDelete {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestAccessDeleteBuilder<crate::generics::MissingUserId> {
        RequestAccessDeleteBuilder {
            body: Default::default(),
            _user_id: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestAccessDeletePostBuilder<crate::generics::MissingUserId> {
        RequestAccessDeletePostBuilder {
            body: Default::default(),
            _user_id: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post_1() -> RequestAccessDeletePostBuilder1<crate::generics::MissingUserId> {
        RequestAccessDeletePostBuilder1 {
            body: Default::default(),
            _user_id: core::marker::PhantomData,
        }
    }
}

impl Into<RequestAccessDelete> for RequestAccessDeleteBuilder<crate::generics::UserIdExists> {
    fn into(self) -> RequestAccessDelete {
        self.body
    }
}

impl Into<RequestAccessDelete> for RequestAccessDeletePostBuilder<crate::generics::UserIdExists> {
    fn into(self) -> RequestAccessDelete {
        self.body
    }
}

impl Into<RequestAccessDelete> for RequestAccessDeletePostBuilder1<crate::generics::UserIdExists> {
    fn into(self) -> RequestAccessDelete {
        self.body
    }
}

/// Builder for [`RequestAccessDelete`](./struct.RequestAccessDelete.html) object.
#[derive(Debug, Clone)]
pub struct RequestAccessDeleteBuilder<UserId> {
    body: self::RequestAccessDelete,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<UserId> RequestAccessDeleteBuilder<UserId> {
    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> RequestAccessDeleteBuilder<crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

/// Builder created by [`RequestAccessDelete::post`](./struct.RequestAccessDelete.html#method.post) method for a `POST` operation associated with `RequestAccessDelete`.
#[derive(Debug, Clone)]
pub struct RequestAccessDeletePostBuilder<UserId> {
    body: self::RequestAccessDelete,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<UserId> RequestAccessDeletePostBuilder<UserId> {
    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> RequestAccessDeletePostBuilder<crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestAccessDeletePostBuilder<crate::generics::UserIdExists> {
    type Output = Any<serde_json::Value>;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/client/access/delete".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body)
        .header(http::header::ACCEPT.as_str(), "application/json"))
    }
}

/// Builder created by [`RequestAccessDelete::post_1`](./struct.RequestAccessDelete.html#method.post_1) method for a `POST` operation associated with `RequestAccessDelete`.
#[derive(Debug, Clone)]
pub struct RequestAccessDeletePostBuilder1<UserId> {
    body: self::RequestAccessDelete,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<UserId> RequestAccessDeletePostBuilder1<UserId> {
    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> RequestAccessDeletePostBuilder1<crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestAccessDeletePostBuilder1<crate::generics::UserIdExists> {
    type Output = Any<serde_json::Value>;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/user/access/delete".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body)
        .header(http::header::ACCEPT.as_str(), "application/json"))
    }
}
