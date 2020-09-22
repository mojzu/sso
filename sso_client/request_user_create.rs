#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestUserCreate {
    /// User email address
    pub email: String,
    /// User enable flag
    pub enable: bool,
    /// User locale
    pub locale: String,
    /// User name
    pub name: String,
    /// User password
    pub password: Option<crate::request_user_create::RequestUserCreatePassword>,
    /// User access scope
    pub scope: String,
    /// User timezone
    pub timezone: String,
}

/// User password
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestUserCreatePassword {
    /// User allow password reset flag
    pub allow_reset: bool,
    /// User password
    pub password: String,
    /// User require password update flag
    pub require_update: bool,
}

impl RequestUserCreate {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestUserCreateBuilder<crate::generics::MissingEmail, crate::generics::MissingEnable, crate::generics::MissingLocale, crate::generics::MissingName, crate::generics::MissingScope, crate::generics::MissingTimezone> {
        RequestUserCreateBuilder {
            body: Default::default(),
            _email: core::marker::PhantomData,
            _enable: core::marker::PhantomData,
            _locale: core::marker::PhantomData,
            _name: core::marker::PhantomData,
            _scope: core::marker::PhantomData,
            _timezone: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestUserCreatePostBuilder<crate::generics::MissingEmail, crate::generics::MissingEnable, crate::generics::MissingLocale, crate::generics::MissingName, crate::generics::MissingScope, crate::generics::MissingTimezone> {
        RequestUserCreatePostBuilder {
            body: Default::default(),
            _email: core::marker::PhantomData,
            _enable: core::marker::PhantomData,
            _locale: core::marker::PhantomData,
            _name: core::marker::PhantomData,
            _scope: core::marker::PhantomData,
            _timezone: core::marker::PhantomData,
        }
    }
}

impl Into<RequestUserCreate> for RequestUserCreateBuilder<crate::generics::EmailExists, crate::generics::EnableExists, crate::generics::LocaleExists, crate::generics::NameExists, crate::generics::ScopeExists, crate::generics::TimezoneExists> {
    fn into(self) -> RequestUserCreate {
        self.body
    }
}

impl Into<RequestUserCreate> for RequestUserCreatePostBuilder<crate::generics::EmailExists, crate::generics::EnableExists, crate::generics::LocaleExists, crate::generics::NameExists, crate::generics::ScopeExists, crate::generics::TimezoneExists> {
    fn into(self) -> RequestUserCreate {
        self.body
    }
}

/// Builder for [`RequestUserCreate`](./struct.RequestUserCreate.html) object.
#[derive(Debug, Clone)]
pub struct RequestUserCreateBuilder<Email, Enable, Locale, Name, Scope, Timezone> {
    body: self::RequestUserCreate,
    _email: core::marker::PhantomData<Email>,
    _enable: core::marker::PhantomData<Enable>,
    _locale: core::marker::PhantomData<Locale>,
    _name: core::marker::PhantomData<Name>,
    _scope: core::marker::PhantomData<Scope>,
    _timezone: core::marker::PhantomData<Timezone>,
}

impl<Email, Enable, Locale, Name, Scope, Timezone> RequestUserCreateBuilder<Email, Enable, Locale, Name, Scope, Timezone> {
    /// User email address
    #[inline]
    pub fn email(mut self, value: impl Into<String>) -> RequestUserCreateBuilder<crate::generics::EmailExists, Enable, Locale, Name, Scope, Timezone> {
        self.body.email = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// User enable flag
    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> RequestUserCreateBuilder<Email, crate::generics::EnableExists, Locale, Name, Scope, Timezone> {
        self.body.enable = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// User locale
    #[inline]
    pub fn locale(mut self, value: impl Into<String>) -> RequestUserCreateBuilder<Email, Enable, crate::generics::LocaleExists, Name, Scope, Timezone> {
        self.body.locale = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// User name
    #[inline]
    pub fn name(mut self, value: impl Into<String>) -> RequestUserCreateBuilder<Email, Enable, Locale, crate::generics::NameExists, Scope, Timezone> {
        self.body.name = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// User password
    #[inline]
    pub fn password(mut self, value: crate::request_user_create::RequestUserCreatePasswordBuilder<crate::generics::AllowResetExists, crate::generics::PasswordExists, crate::generics::RequireUpdateExists>) -> Self {
        self.body.password = Some(value.into());
        self
    }

    /// User access scope
    #[inline]
    pub fn scope(mut self, value: impl Into<String>) -> RequestUserCreateBuilder<Email, Enable, Locale, Name, crate::generics::ScopeExists, Timezone> {
        self.body.scope = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// User timezone
    #[inline]
    pub fn timezone(mut self, value: impl Into<String>) -> RequestUserCreateBuilder<Email, Enable, Locale, Name, Scope, crate::generics::TimezoneExists> {
        self.body.timezone = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

/// Builder created by [`RequestUserCreate::post`](./struct.RequestUserCreate.html#method.post) method for a `POST` operation associated with `RequestUserCreate`.
#[derive(Debug, Clone)]
pub struct RequestUserCreatePostBuilder<Email, Enable, Locale, Name, Scope, Timezone> {
    body: self::RequestUserCreate,
    _email: core::marker::PhantomData<Email>,
    _enable: core::marker::PhantomData<Enable>,
    _locale: core::marker::PhantomData<Locale>,
    _name: core::marker::PhantomData<Name>,
    _scope: core::marker::PhantomData<Scope>,
    _timezone: core::marker::PhantomData<Timezone>,
}

impl<Email, Enable, Locale, Name, Scope, Timezone> RequestUserCreatePostBuilder<Email, Enable, Locale, Name, Scope, Timezone> {
    /// User email address
    #[inline]
    pub fn email(mut self, value: impl Into<String>) -> RequestUserCreatePostBuilder<crate::generics::EmailExists, Enable, Locale, Name, Scope, Timezone> {
        self.body.email = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// User enable flag
    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> RequestUserCreatePostBuilder<Email, crate::generics::EnableExists, Locale, Name, Scope, Timezone> {
        self.body.enable = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// User locale
    #[inline]
    pub fn locale(mut self, value: impl Into<String>) -> RequestUserCreatePostBuilder<Email, Enable, crate::generics::LocaleExists, Name, Scope, Timezone> {
        self.body.locale = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// User name
    #[inline]
    pub fn name(mut self, value: impl Into<String>) -> RequestUserCreatePostBuilder<Email, Enable, Locale, crate::generics::NameExists, Scope, Timezone> {
        self.body.name = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// User password
    #[inline]
    pub fn password(mut self, value: crate::request_user_create::RequestUserCreatePasswordBuilder<crate::generics::AllowResetExists, crate::generics::PasswordExists, crate::generics::RequireUpdateExists>) -> Self {
        self.body.password = Some(value.into());
        self
    }

    /// User access scope
    #[inline]
    pub fn scope(mut self, value: impl Into<String>) -> RequestUserCreatePostBuilder<Email, Enable, Locale, Name, crate::generics::ScopeExists, Timezone> {
        self.body.scope = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// User timezone
    #[inline]
    pub fn timezone(mut self, value: impl Into<String>) -> RequestUserCreatePostBuilder<Email, Enable, Locale, Name, Scope, crate::generics::TimezoneExists> {
        self.body.timezone = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestUserCreatePostBuilder<crate::generics::EmailExists, crate::generics::EnableExists, crate::generics::LocaleExists, crate::generics::NameExists, crate::generics::ScopeExists, crate::generics::TimezoneExists> {
    type Output = crate::response_user::ResponseUser;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/user/create".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body))
    }
}

impl RequestUserCreatePassword {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestUserCreatePasswordBuilder<crate::generics::MissingAllowReset, crate::generics::MissingPassword, crate::generics::MissingRequireUpdate> {
        RequestUserCreatePasswordBuilder {
            body: Default::default(),
            _allow_reset: core::marker::PhantomData,
            _password: core::marker::PhantomData,
            _require_update: core::marker::PhantomData,
        }
    }
}

impl Into<RequestUserCreatePassword> for RequestUserCreatePasswordBuilder<crate::generics::AllowResetExists, crate::generics::PasswordExists, crate::generics::RequireUpdateExists> {
    fn into(self) -> RequestUserCreatePassword {
        self.body
    }
}

/// Builder for [`RequestUserCreatePassword`](./struct.RequestUserCreatePassword.html) object.
#[derive(Debug, Clone)]
pub struct RequestUserCreatePasswordBuilder<AllowReset, Password, RequireUpdate> {
    body: self::RequestUserCreatePassword,
    _allow_reset: core::marker::PhantomData<AllowReset>,
    _password: core::marker::PhantomData<Password>,
    _require_update: core::marker::PhantomData<RequireUpdate>,
}

impl<AllowReset, Password, RequireUpdate> RequestUserCreatePasswordBuilder<AllowReset, Password, RequireUpdate> {
    /// User allow password reset flag
    #[inline]
    pub fn allow_reset(mut self, value: impl Into<bool>) -> RequestUserCreatePasswordBuilder<crate::generics::AllowResetExists, Password, RequireUpdate> {
        self.body.allow_reset = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// User password
    #[inline]
    pub fn password(mut self, value: impl Into<String>) -> RequestUserCreatePasswordBuilder<AllowReset, crate::generics::PasswordExists, RequireUpdate> {
        self.body.password = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// User require password update flag
    #[inline]
    pub fn require_update(mut self, value: impl Into<bool>) -> RequestUserCreatePasswordBuilder<AllowReset, Password, crate::generics::RequireUpdateExists> {
        self.body.require_update = value.into();
        unsafe { std::mem::transmute(self) }
    }
}
