#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestAccessUpdate {
    pub enable: bool,
    pub scope: String,
    pub user_id: String,
}

impl RequestAccessUpdate {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestAccessUpdateBuilder<crate::generics::MissingEnable, crate::generics::MissingScope, crate::generics::MissingUserId> {
        RequestAccessUpdateBuilder {
            body: Default::default(),
            _enable: core::marker::PhantomData,
            _scope: core::marker::PhantomData,
            _user_id: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestAccessUpdatePostBuilder<crate::generics::MissingEnable, crate::generics::MissingScope, crate::generics::MissingUserId> {
        RequestAccessUpdatePostBuilder {
            body: Default::default(),
            _enable: core::marker::PhantomData,
            _scope: core::marker::PhantomData,
            _user_id: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post_1() -> RequestAccessUpdatePostBuilder1<crate::generics::MissingEnable, crate::generics::MissingScope, crate::generics::MissingUserId> {
        RequestAccessUpdatePostBuilder1 {
            body: Default::default(),
            _enable: core::marker::PhantomData,
            _scope: core::marker::PhantomData,
            _user_id: core::marker::PhantomData,
        }
    }
}

impl Into<RequestAccessUpdate> for RequestAccessUpdateBuilder<crate::generics::EnableExists, crate::generics::ScopeExists, crate::generics::UserIdExists> {
    fn into(self) -> RequestAccessUpdate {
        self.body
    }
}

impl Into<RequestAccessUpdate> for RequestAccessUpdatePostBuilder<crate::generics::EnableExists, crate::generics::ScopeExists, crate::generics::UserIdExists> {
    fn into(self) -> RequestAccessUpdate {
        self.body
    }
}

impl Into<RequestAccessUpdate> for RequestAccessUpdatePostBuilder1<crate::generics::EnableExists, crate::generics::ScopeExists, crate::generics::UserIdExists> {
    fn into(self) -> RequestAccessUpdate {
        self.body
    }
}

/// Builder for [`RequestAccessUpdate`](./struct.RequestAccessUpdate.html) object.
#[derive(Debug, Clone)]
pub struct RequestAccessUpdateBuilder<Enable, Scope, UserId> {
    body: self::RequestAccessUpdate,
    _enable: core::marker::PhantomData<Enable>,
    _scope: core::marker::PhantomData<Scope>,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<Enable, Scope, UserId> RequestAccessUpdateBuilder<Enable, Scope, UserId> {
    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> RequestAccessUpdateBuilder<crate::generics::EnableExists, Scope, UserId> {
        self.body.enable = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn scope(mut self, value: impl Into<String>) -> RequestAccessUpdateBuilder<Enable, crate::generics::ScopeExists, UserId> {
        self.body.scope = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> RequestAccessUpdateBuilder<Enable, Scope, crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

/// Builder created by [`RequestAccessUpdate::post`](./struct.RequestAccessUpdate.html#method.post) method for a `POST` operation associated with `RequestAccessUpdate`.
#[derive(Debug, Clone)]
pub struct RequestAccessUpdatePostBuilder<Enable, Scope, UserId> {
    body: self::RequestAccessUpdate,
    _enable: core::marker::PhantomData<Enable>,
    _scope: core::marker::PhantomData<Scope>,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<Enable, Scope, UserId> RequestAccessUpdatePostBuilder<Enable, Scope, UserId> {
    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> RequestAccessUpdatePostBuilder<crate::generics::EnableExists, Scope, UserId> {
        self.body.enable = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn scope(mut self, value: impl Into<String>) -> RequestAccessUpdatePostBuilder<Enable, crate::generics::ScopeExists, UserId> {
        self.body.scope = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> RequestAccessUpdatePostBuilder<Enable, Scope, crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestAccessUpdatePostBuilder<crate::generics::EnableExists, crate::generics::ScopeExists, crate::generics::UserIdExists> {
    type Output = crate::response_access::ResponseAccess;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/client/access/update".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body))
    }
}

/// Builder created by [`RequestAccessUpdate::post_1`](./struct.RequestAccessUpdate.html#method.post_1) method for a `POST` operation associated with `RequestAccessUpdate`.
#[derive(Debug, Clone)]
pub struct RequestAccessUpdatePostBuilder1<Enable, Scope, UserId> {
    body: self::RequestAccessUpdate,
    _enable: core::marker::PhantomData<Enable>,
    _scope: core::marker::PhantomData<Scope>,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<Enable, Scope, UserId> RequestAccessUpdatePostBuilder1<Enable, Scope, UserId> {
    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> RequestAccessUpdatePostBuilder1<crate::generics::EnableExists, Scope, UserId> {
        self.body.enable = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn scope(mut self, value: impl Into<String>) -> RequestAccessUpdatePostBuilder1<Enable, crate::generics::ScopeExists, UserId> {
        self.body.scope = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> RequestAccessUpdatePostBuilder1<Enable, Scope, crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestAccessUpdatePostBuilder1<crate::generics::EnableExists, crate::generics::ScopeExists, crate::generics::UserIdExists> {
    type Output = crate::response_access::ResponseAccess;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/user/access/update".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body))
    }
}
