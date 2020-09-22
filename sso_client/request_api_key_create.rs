#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestApiKeyCreate {
    pub enable: bool,
    pub name: String,
    pub scope: String,
    pub user_id: String,
}

impl RequestApiKeyCreate {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestApiKeyCreateBuilder<crate::generics::MissingEnable, crate::generics::MissingName, crate::generics::MissingScope, crate::generics::MissingUserId> {
        RequestApiKeyCreateBuilder {
            body: Default::default(),
            _enable: core::marker::PhantomData,
            _name: core::marker::PhantomData,
            _scope: core::marker::PhantomData,
            _user_id: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestApiKeyCreatePostBuilder<crate::generics::MissingEnable, crate::generics::MissingName, crate::generics::MissingScope, crate::generics::MissingUserId> {
        RequestApiKeyCreatePostBuilder {
            body: Default::default(),
            _enable: core::marker::PhantomData,
            _name: core::marker::PhantomData,
            _scope: core::marker::PhantomData,
            _user_id: core::marker::PhantomData,
        }
    }
}

impl Into<RequestApiKeyCreate> for RequestApiKeyCreateBuilder<crate::generics::EnableExists, crate::generics::NameExists, crate::generics::ScopeExists, crate::generics::UserIdExists> {
    fn into(self) -> RequestApiKeyCreate {
        self.body
    }
}

impl Into<RequestApiKeyCreate> for RequestApiKeyCreatePostBuilder<crate::generics::EnableExists, crate::generics::NameExists, crate::generics::ScopeExists, crate::generics::UserIdExists> {
    fn into(self) -> RequestApiKeyCreate {
        self.body
    }
}

/// Builder for [`RequestApiKeyCreate`](./struct.RequestApiKeyCreate.html) object.
#[derive(Debug, Clone)]
pub struct RequestApiKeyCreateBuilder<Enable, Name, Scope, UserId> {
    body: self::RequestApiKeyCreate,
    _enable: core::marker::PhantomData<Enable>,
    _name: core::marker::PhantomData<Name>,
    _scope: core::marker::PhantomData<Scope>,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<Enable, Name, Scope, UserId> RequestApiKeyCreateBuilder<Enable, Name, Scope, UserId> {
    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> RequestApiKeyCreateBuilder<crate::generics::EnableExists, Name, Scope, UserId> {
        self.body.enable = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn name(mut self, value: impl Into<String>) -> RequestApiKeyCreateBuilder<Enable, crate::generics::NameExists, Scope, UserId> {
        self.body.name = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn scope(mut self, value: impl Into<String>) -> RequestApiKeyCreateBuilder<Enable, Name, crate::generics::ScopeExists, UserId> {
        self.body.scope = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> RequestApiKeyCreateBuilder<Enable, Name, Scope, crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

/// Builder created by [`RequestApiKeyCreate::post`](./struct.RequestApiKeyCreate.html#method.post) method for a `POST` operation associated with `RequestApiKeyCreate`.
#[derive(Debug, Clone)]
pub struct RequestApiKeyCreatePostBuilder<Enable, Name, Scope, UserId> {
    body: self::RequestApiKeyCreate,
    _enable: core::marker::PhantomData<Enable>,
    _name: core::marker::PhantomData<Name>,
    _scope: core::marker::PhantomData<Scope>,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<Enable, Name, Scope, UserId> RequestApiKeyCreatePostBuilder<Enable, Name, Scope, UserId> {
    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> RequestApiKeyCreatePostBuilder<crate::generics::EnableExists, Name, Scope, UserId> {
        self.body.enable = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn name(mut self, value: impl Into<String>) -> RequestApiKeyCreatePostBuilder<Enable, crate::generics::NameExists, Scope, UserId> {
        self.body.name = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn scope(mut self, value: impl Into<String>) -> RequestApiKeyCreatePostBuilder<Enable, Name, crate::generics::ScopeExists, UserId> {
        self.body.scope = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> RequestApiKeyCreatePostBuilder<Enable, Name, Scope, crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestApiKeyCreatePostBuilder<crate::generics::EnableExists, crate::generics::NameExists, crate::generics::ScopeExists, crate::generics::UserIdExists> {
    type Output = crate::response_api_key::ResponseApiKey;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/user/api-key/create".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body))
    }
}
