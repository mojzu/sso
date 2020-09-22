#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseClient {
    pub enable: bool,
    pub id: String,
    pub name: String,
    pub redirect_uri: String,
    pub register_enable: bool,
    pub register_scope: String,
    pub scope: String,
    pub uri: String,
    pub user_scope: String,
}

impl ResponseClient {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseClientBuilder<crate::generics::MissingEnable, crate::generics::MissingId, crate::generics::MissingName, crate::generics::MissingRedirectUri, crate::generics::MissingRegisterEnable, crate::generics::MissingRegisterScope, crate::generics::MissingScope, crate::generics::MissingUri, crate::generics::MissingUserScope> {
        ResponseClientBuilder {
            body: Default::default(),
            _enable: core::marker::PhantomData,
            _id: core::marker::PhantomData,
            _name: core::marker::PhantomData,
            _redirect_uri: core::marker::PhantomData,
            _register_enable: core::marker::PhantomData,
            _register_scope: core::marker::PhantomData,
            _scope: core::marker::PhantomData,
            _uri: core::marker::PhantomData,
            _user_scope: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> ResponseClientPostBuilder {
        ResponseClientPostBuilder
    }
}

impl Into<ResponseClient> for ResponseClientBuilder<crate::generics::EnableExists, crate::generics::IdExists, crate::generics::NameExists, crate::generics::RedirectUriExists, crate::generics::RegisterEnableExists, crate::generics::RegisterScopeExists, crate::generics::ScopeExists, crate::generics::UriExists, crate::generics::UserScopeExists> {
    fn into(self) -> ResponseClient {
        self.body
    }
}

/// Builder for [`ResponseClient`](./struct.ResponseClient.html) object.
#[derive(Debug, Clone)]
pub struct ResponseClientBuilder<Enable, Id, Name, RedirectUri, RegisterEnable, RegisterScope, Scope, Uri, UserScope> {
    body: self::ResponseClient,
    _enable: core::marker::PhantomData<Enable>,
    _id: core::marker::PhantomData<Id>,
    _name: core::marker::PhantomData<Name>,
    _redirect_uri: core::marker::PhantomData<RedirectUri>,
    _register_enable: core::marker::PhantomData<RegisterEnable>,
    _register_scope: core::marker::PhantomData<RegisterScope>,
    _scope: core::marker::PhantomData<Scope>,
    _uri: core::marker::PhantomData<Uri>,
    _user_scope: core::marker::PhantomData<UserScope>,
}

impl<Enable, Id, Name, RedirectUri, RegisterEnable, RegisterScope, Scope, Uri, UserScope> ResponseClientBuilder<Enable, Id, Name, RedirectUri, RegisterEnable, RegisterScope, Scope, Uri, UserScope> {
    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> ResponseClientBuilder<crate::generics::EnableExists, Id, Name, RedirectUri, RegisterEnable, RegisterScope, Scope, Uri, UserScope> {
        self.body.enable = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn id(mut self, value: impl Into<String>) -> ResponseClientBuilder<Enable, crate::generics::IdExists, Name, RedirectUri, RegisterEnable, RegisterScope, Scope, Uri, UserScope> {
        self.body.id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn name(mut self, value: impl Into<String>) -> ResponseClientBuilder<Enable, Id, crate::generics::NameExists, RedirectUri, RegisterEnable, RegisterScope, Scope, Uri, UserScope> {
        self.body.name = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn redirect_uri(mut self, value: impl Into<String>) -> ResponseClientBuilder<Enable, Id, Name, crate::generics::RedirectUriExists, RegisterEnable, RegisterScope, Scope, Uri, UserScope> {
        self.body.redirect_uri = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn register_enable(mut self, value: impl Into<bool>) -> ResponseClientBuilder<Enable, Id, Name, RedirectUri, crate::generics::RegisterEnableExists, RegisterScope, Scope, Uri, UserScope> {
        self.body.register_enable = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn register_scope(mut self, value: impl Into<String>) -> ResponseClientBuilder<Enable, Id, Name, RedirectUri, RegisterEnable, crate::generics::RegisterScopeExists, Scope, Uri, UserScope> {
        self.body.register_scope = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn scope(mut self, value: impl Into<String>) -> ResponseClientBuilder<Enable, Id, Name, RedirectUri, RegisterEnable, RegisterScope, crate::generics::ScopeExists, Uri, UserScope> {
        self.body.scope = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn uri(mut self, value: impl Into<String>) -> ResponseClientBuilder<Enable, Id, Name, RedirectUri, RegisterEnable, RegisterScope, Scope, crate::generics::UriExists, UserScope> {
        self.body.uri = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn user_scope(mut self, value: impl Into<String>) -> ResponseClientBuilder<Enable, Id, Name, RedirectUri, RegisterEnable, RegisterScope, Scope, Uri, crate::generics::UserScopeExists> {
        self.body.user_scope = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

/// Builder created by [`ResponseClient::post`](./struct.ResponseClient.html#method.post) method for a `POST` operation associated with `ResponseClient`.
#[derive(Debug, Clone)]
pub struct ResponseClientPostBuilder;


impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for ResponseClientPostBuilder {
    type Output = ResponseClient;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/client/read".into()
    }
}
