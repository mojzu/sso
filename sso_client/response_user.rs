#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseUser {
    pub access: Option<crate::response_user::ResponseUserAccess>,
    pub created_at: String,
    pub email: String,
    pub enable: bool,
    pub id: String,
    pub locale: String,
    pub name: String,
    pub oauth2_provider: Vec<crate::response_user::ResponseUserOauth2ProviderItem>,
    pub oauth2_provider_count: i64,
    pub password: Option<crate::response_user::ResponseUserPassword>,
    #[serde(rename = "static")]
    pub static_: bool,
    pub timezone: String,
    pub updated_at: String,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseUserAccess {
    pub client_id: String,
    pub created_at: String,
    pub enable: bool,
    pub scope: String,
    #[serde(rename = "static")]
    pub static_: bool,
    pub updated_at: String,
    pub user_id: String,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseUserOauth2ProviderItem {
    pub created_at: String,
    pub oauth2_provider: String,
    #[serde(rename = "static")]
    pub static_: bool,
    pub sub: String,
    pub user_id: String,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseUserPassword {
    pub allow_reset: bool,
    pub created_at: String,
    pub require_update: bool,
    #[serde(rename = "static")]
    pub static_: bool,
    pub updated_at: String,
    pub user_id: String,
}

impl ResponseUser {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseUserBuilder<crate::generics::MissingCreatedAt, crate::generics::MissingEmail, crate::generics::MissingEnable, crate::generics::MissingId, crate::generics::MissingLocale, crate::generics::MissingName, crate::generics::MissingOauth2Provider, crate::generics::MissingOauth2ProviderCount, crate::generics::MissingStatic, crate::generics::MissingTimezone, crate::generics::MissingUpdatedAt> {
        ResponseUserBuilder {
            body: Default::default(),
            _created_at: core::marker::PhantomData,
            _email: core::marker::PhantomData,
            _enable: core::marker::PhantomData,
            _id: core::marker::PhantomData,
            _locale: core::marker::PhantomData,
            _name: core::marker::PhantomData,
            _oauth2_provider: core::marker::PhantomData,
            _oauth2_provider_count: core::marker::PhantomData,
            _static: core::marker::PhantomData,
            _timezone: core::marker::PhantomData,
            _updated_at: core::marker::PhantomData,
        }
    }
}

impl Into<ResponseUser> for ResponseUserBuilder<crate::generics::CreatedAtExists, crate::generics::EmailExists, crate::generics::EnableExists, crate::generics::IdExists, crate::generics::LocaleExists, crate::generics::NameExists, crate::generics::Oauth2ProviderExists, crate::generics::Oauth2ProviderCountExists, crate::generics::StaticExists, crate::generics::TimezoneExists, crate::generics::UpdatedAtExists> {
    fn into(self) -> ResponseUser {
        self.body
    }
}

/// Builder for [`ResponseUser`](./struct.ResponseUser.html) object.
#[derive(Debug, Clone)]
pub struct ResponseUserBuilder<CreatedAt, Email, Enable, Id, Locale, Name, Oauth2Provider, Oauth2ProviderCount, Static, Timezone, UpdatedAt> {
    body: self::ResponseUser,
    _created_at: core::marker::PhantomData<CreatedAt>,
    _email: core::marker::PhantomData<Email>,
    _enable: core::marker::PhantomData<Enable>,
    _id: core::marker::PhantomData<Id>,
    _locale: core::marker::PhantomData<Locale>,
    _name: core::marker::PhantomData<Name>,
    _oauth2_provider: core::marker::PhantomData<Oauth2Provider>,
    _oauth2_provider_count: core::marker::PhantomData<Oauth2ProviderCount>,
    _static: core::marker::PhantomData<Static>,
    _timezone: core::marker::PhantomData<Timezone>,
    _updated_at: core::marker::PhantomData<UpdatedAt>,
}

impl<CreatedAt, Email, Enable, Id, Locale, Name, Oauth2Provider, Oauth2ProviderCount, Static, Timezone, UpdatedAt> ResponseUserBuilder<CreatedAt, Email, Enable, Id, Locale, Name, Oauth2Provider, Oauth2ProviderCount, Static, Timezone, UpdatedAt> {
    #[inline]
    pub fn access(mut self, value: crate::response_user::ResponseUserAccessBuilder<crate::generics::ClientIdExists, crate::generics::CreatedAtExists, crate::generics::EnableExists, crate::generics::ScopeExists, crate::generics::StaticExists, crate::generics::UpdatedAtExists, crate::generics::UserIdExists>) -> Self {
        self.body.access = Some(value.into());
        self
    }

    #[inline]
    pub fn created_at(mut self, value: impl Into<String>) -> ResponseUserBuilder<crate::generics::CreatedAtExists, Email, Enable, Id, Locale, Name, Oauth2Provider, Oauth2ProviderCount, Static, Timezone, UpdatedAt> {
        self.body.created_at = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn email(mut self, value: impl Into<String>) -> ResponseUserBuilder<CreatedAt, crate::generics::EmailExists, Enable, Id, Locale, Name, Oauth2Provider, Oauth2ProviderCount, Static, Timezone, UpdatedAt> {
        self.body.email = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> ResponseUserBuilder<CreatedAt, Email, crate::generics::EnableExists, Id, Locale, Name, Oauth2Provider, Oauth2ProviderCount, Static, Timezone, UpdatedAt> {
        self.body.enable = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn id(mut self, value: impl Into<String>) -> ResponseUserBuilder<CreatedAt, Email, Enable, crate::generics::IdExists, Locale, Name, Oauth2Provider, Oauth2ProviderCount, Static, Timezone, UpdatedAt> {
        self.body.id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn locale(mut self, value: impl Into<String>) -> ResponseUserBuilder<CreatedAt, Email, Enable, Id, crate::generics::LocaleExists, Name, Oauth2Provider, Oauth2ProviderCount, Static, Timezone, UpdatedAt> {
        self.body.locale = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn name(mut self, value: impl Into<String>) -> ResponseUserBuilder<CreatedAt, Email, Enable, Id, Locale, crate::generics::NameExists, Oauth2Provider, Oauth2ProviderCount, Static, Timezone, UpdatedAt> {
        self.body.name = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn oauth2_provider(mut self, value: impl Iterator<Item = crate::response_user::ResponseUserOauth2ProviderItemBuilder<crate::generics::CreatedAtExists, crate::generics::Oauth2ProviderExists, crate::generics::StaticExists, crate::generics::SubExists, crate::generics::UserIdExists>>) -> ResponseUserBuilder<CreatedAt, Email, Enable, Id, Locale, Name, crate::generics::Oauth2ProviderExists, Oauth2ProviderCount, Static, Timezone, UpdatedAt> {
        self.body.oauth2_provider = value.map(|value| value.into()).collect::<Vec<_>>().into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn oauth2_provider_count(mut self, value: impl Into<i64>) -> ResponseUserBuilder<CreatedAt, Email, Enable, Id, Locale, Name, Oauth2Provider, crate::generics::Oauth2ProviderCountExists, Static, Timezone, UpdatedAt> {
        self.body.oauth2_provider_count = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn password(mut self, value: crate::response_user::ResponseUserPasswordBuilder<crate::generics::AllowResetExists, crate::generics::CreatedAtExists, crate::generics::RequireUpdateExists, crate::generics::StaticExists, crate::generics::UpdatedAtExists, crate::generics::UserIdExists>) -> Self {
        self.body.password = Some(value.into());
        self
    }

    #[inline]
    pub fn static_(mut self, value: impl Into<bool>) -> ResponseUserBuilder<CreatedAt, Email, Enable, Id, Locale, Name, Oauth2Provider, Oauth2ProviderCount, crate::generics::StaticExists, Timezone, UpdatedAt> {
        self.body.static_ = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn timezone(mut self, value: impl Into<String>) -> ResponseUserBuilder<CreatedAt, Email, Enable, Id, Locale, Name, Oauth2Provider, Oauth2ProviderCount, Static, crate::generics::TimezoneExists, UpdatedAt> {
        self.body.timezone = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn updated_at(mut self, value: impl Into<String>) -> ResponseUserBuilder<CreatedAt, Email, Enable, Id, Locale, Name, Oauth2Provider, Oauth2ProviderCount, Static, Timezone, crate::generics::UpdatedAtExists> {
        self.body.updated_at = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl ResponseUserAccess {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseUserAccessBuilder<crate::generics::MissingClientId, crate::generics::MissingCreatedAt, crate::generics::MissingEnable, crate::generics::MissingScope, crate::generics::MissingStatic, crate::generics::MissingUpdatedAt, crate::generics::MissingUserId> {
        ResponseUserAccessBuilder {
            body: Default::default(),
            _client_id: core::marker::PhantomData,
            _created_at: core::marker::PhantomData,
            _enable: core::marker::PhantomData,
            _scope: core::marker::PhantomData,
            _static: core::marker::PhantomData,
            _updated_at: core::marker::PhantomData,
            _user_id: core::marker::PhantomData,
        }
    }
}

impl Into<ResponseUserAccess> for ResponseUserAccessBuilder<crate::generics::ClientIdExists, crate::generics::CreatedAtExists, crate::generics::EnableExists, crate::generics::ScopeExists, crate::generics::StaticExists, crate::generics::UpdatedAtExists, crate::generics::UserIdExists> {
    fn into(self) -> ResponseUserAccess {
        self.body
    }
}

/// Builder for [`ResponseUserAccess`](./struct.ResponseUserAccess.html) object.
#[derive(Debug, Clone)]
pub struct ResponseUserAccessBuilder<ClientId, CreatedAt, Enable, Scope, Static, UpdatedAt, UserId> {
    body: self::ResponseUserAccess,
    _client_id: core::marker::PhantomData<ClientId>,
    _created_at: core::marker::PhantomData<CreatedAt>,
    _enable: core::marker::PhantomData<Enable>,
    _scope: core::marker::PhantomData<Scope>,
    _static: core::marker::PhantomData<Static>,
    _updated_at: core::marker::PhantomData<UpdatedAt>,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<ClientId, CreatedAt, Enable, Scope, Static, UpdatedAt, UserId> ResponseUserAccessBuilder<ClientId, CreatedAt, Enable, Scope, Static, UpdatedAt, UserId> {
    #[inline]
    pub fn client_id(mut self, value: impl Into<String>) -> ResponseUserAccessBuilder<crate::generics::ClientIdExists, CreatedAt, Enable, Scope, Static, UpdatedAt, UserId> {
        self.body.client_id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn created_at(mut self, value: impl Into<String>) -> ResponseUserAccessBuilder<ClientId, crate::generics::CreatedAtExists, Enable, Scope, Static, UpdatedAt, UserId> {
        self.body.created_at = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> ResponseUserAccessBuilder<ClientId, CreatedAt, crate::generics::EnableExists, Scope, Static, UpdatedAt, UserId> {
        self.body.enable = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn scope(mut self, value: impl Into<String>) -> ResponseUserAccessBuilder<ClientId, CreatedAt, Enable, crate::generics::ScopeExists, Static, UpdatedAt, UserId> {
        self.body.scope = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn static_(mut self, value: impl Into<bool>) -> ResponseUserAccessBuilder<ClientId, CreatedAt, Enable, Scope, crate::generics::StaticExists, UpdatedAt, UserId> {
        self.body.static_ = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn updated_at(mut self, value: impl Into<String>) -> ResponseUserAccessBuilder<ClientId, CreatedAt, Enable, Scope, Static, crate::generics::UpdatedAtExists, UserId> {
        self.body.updated_at = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> ResponseUserAccessBuilder<ClientId, CreatedAt, Enable, Scope, Static, UpdatedAt, crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl ResponseUserOauth2ProviderItem {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseUserOauth2ProviderItemBuilder<crate::generics::MissingCreatedAt, crate::generics::MissingOauth2Provider, crate::generics::MissingStatic, crate::generics::MissingSub, crate::generics::MissingUserId> {
        ResponseUserOauth2ProviderItemBuilder {
            body: Default::default(),
            _created_at: core::marker::PhantomData,
            _oauth2_provider: core::marker::PhantomData,
            _static: core::marker::PhantomData,
            _sub: core::marker::PhantomData,
            _user_id: core::marker::PhantomData,
        }
    }
}

impl Into<ResponseUserOauth2ProviderItem> for ResponseUserOauth2ProviderItemBuilder<crate::generics::CreatedAtExists, crate::generics::Oauth2ProviderExists, crate::generics::StaticExists, crate::generics::SubExists, crate::generics::UserIdExists> {
    fn into(self) -> ResponseUserOauth2ProviderItem {
        self.body
    }
}

/// Builder for [`ResponseUserOauth2ProviderItem`](./struct.ResponseUserOauth2ProviderItem.html) object.
#[derive(Debug, Clone)]
pub struct ResponseUserOauth2ProviderItemBuilder<CreatedAt, Oauth2Provider, Static, Sub, UserId> {
    body: self::ResponseUserOauth2ProviderItem,
    _created_at: core::marker::PhantomData<CreatedAt>,
    _oauth2_provider: core::marker::PhantomData<Oauth2Provider>,
    _static: core::marker::PhantomData<Static>,
    _sub: core::marker::PhantomData<Sub>,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<CreatedAt, Oauth2Provider, Static, Sub, UserId> ResponseUserOauth2ProviderItemBuilder<CreatedAt, Oauth2Provider, Static, Sub, UserId> {
    #[inline]
    pub fn created_at(mut self, value: impl Into<String>) -> ResponseUserOauth2ProviderItemBuilder<crate::generics::CreatedAtExists, Oauth2Provider, Static, Sub, UserId> {
        self.body.created_at = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn oauth2_provider(mut self, value: impl Into<String>) -> ResponseUserOauth2ProviderItemBuilder<CreatedAt, crate::generics::Oauth2ProviderExists, Static, Sub, UserId> {
        self.body.oauth2_provider = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn static_(mut self, value: impl Into<bool>) -> ResponseUserOauth2ProviderItemBuilder<CreatedAt, Oauth2Provider, crate::generics::StaticExists, Sub, UserId> {
        self.body.static_ = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn sub(mut self, value: impl Into<String>) -> ResponseUserOauth2ProviderItemBuilder<CreatedAt, Oauth2Provider, Static, crate::generics::SubExists, UserId> {
        self.body.sub = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> ResponseUserOauth2ProviderItemBuilder<CreatedAt, Oauth2Provider, Static, Sub, crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}

impl ResponseUserPassword {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseUserPasswordBuilder<crate::generics::MissingAllowReset, crate::generics::MissingCreatedAt, crate::generics::MissingRequireUpdate, crate::generics::MissingStatic, crate::generics::MissingUpdatedAt, crate::generics::MissingUserId> {
        ResponseUserPasswordBuilder {
            body: Default::default(),
            _allow_reset: core::marker::PhantomData,
            _created_at: core::marker::PhantomData,
            _require_update: core::marker::PhantomData,
            _static: core::marker::PhantomData,
            _updated_at: core::marker::PhantomData,
            _user_id: core::marker::PhantomData,
        }
    }
}

impl Into<ResponseUserPassword> for ResponseUserPasswordBuilder<crate::generics::AllowResetExists, crate::generics::CreatedAtExists, crate::generics::RequireUpdateExists, crate::generics::StaticExists, crate::generics::UpdatedAtExists, crate::generics::UserIdExists> {
    fn into(self) -> ResponseUserPassword {
        self.body
    }
}

/// Builder for [`ResponseUserPassword`](./struct.ResponseUserPassword.html) object.
#[derive(Debug, Clone)]
pub struct ResponseUserPasswordBuilder<AllowReset, CreatedAt, RequireUpdate, Static, UpdatedAt, UserId> {
    body: self::ResponseUserPassword,
    _allow_reset: core::marker::PhantomData<AllowReset>,
    _created_at: core::marker::PhantomData<CreatedAt>,
    _require_update: core::marker::PhantomData<RequireUpdate>,
    _static: core::marker::PhantomData<Static>,
    _updated_at: core::marker::PhantomData<UpdatedAt>,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<AllowReset, CreatedAt, RequireUpdate, Static, UpdatedAt, UserId> ResponseUserPasswordBuilder<AllowReset, CreatedAt, RequireUpdate, Static, UpdatedAt, UserId> {
    #[inline]
    pub fn allow_reset(mut self, value: impl Into<bool>) -> ResponseUserPasswordBuilder<crate::generics::AllowResetExists, CreatedAt, RequireUpdate, Static, UpdatedAt, UserId> {
        self.body.allow_reset = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn created_at(mut self, value: impl Into<String>) -> ResponseUserPasswordBuilder<AllowReset, crate::generics::CreatedAtExists, RequireUpdate, Static, UpdatedAt, UserId> {
        self.body.created_at = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn require_update(mut self, value: impl Into<bool>) -> ResponseUserPasswordBuilder<AllowReset, CreatedAt, crate::generics::RequireUpdateExists, Static, UpdatedAt, UserId> {
        self.body.require_update = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn static_(mut self, value: impl Into<bool>) -> ResponseUserPasswordBuilder<AllowReset, CreatedAt, RequireUpdate, crate::generics::StaticExists, UpdatedAt, UserId> {
        self.body.static_ = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn updated_at(mut self, value: impl Into<String>) -> ResponseUserPasswordBuilder<AllowReset, CreatedAt, RequireUpdate, Static, crate::generics::UpdatedAtExists, UserId> {
        self.body.updated_at = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> ResponseUserPasswordBuilder<AllowReset, CreatedAt, RequireUpdate, Static, UpdatedAt, crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}
