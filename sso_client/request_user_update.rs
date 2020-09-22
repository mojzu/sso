#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestUserUpdate {
    pub access: Option<crate::request_user_update::RequestUserUpdateAccess>,
    pub email: Option<String>,
    pub enable: Option<bool>,
    pub id: String,
    pub locale: Option<String>,
    pub name: Option<String>,
    pub password: Option<crate::request_user_update::RequestUserUpdatePassword>,
    pub timezone: Option<String>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestUserUpdateAccess {
    pub enable: Option<bool>,
    pub scope: Option<String>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestUserUpdatePassword {
    pub allow_reset: Option<bool>,
    pub require_update: Option<bool>,
}

impl RequestUserUpdate {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestUserUpdateBuilder<crate::generics::MissingId> {
        RequestUserUpdateBuilder {
            body: Default::default(),
            _id: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestUserUpdatePostBuilder<crate::generics::MissingId> {
        RequestUserUpdatePostBuilder {
            body: Default::default(),
            _id: core::marker::PhantomData,
        }
    }
}

impl Into<RequestUserUpdate> for RequestUserUpdateBuilder<crate::generics::IdExists> {
    fn into(self) -> RequestUserUpdate {
        self.body
    }
}

impl Into<RequestUserUpdate> for RequestUserUpdatePostBuilder<crate::generics::IdExists> {
    fn into(self) -> RequestUserUpdate {
        self.body
    }
}

/// Builder for [`RequestUserUpdate`](./struct.RequestUserUpdate.html) object.
#[derive(Debug, Clone)]
pub struct RequestUserUpdateBuilder<Id> {
    body: self::RequestUserUpdate,
    _id: core::marker::PhantomData<Id>,
}

impl<Id> RequestUserUpdateBuilder<Id> {
    #[inline]
    pub fn access(mut self, value: crate::request_user_update::RequestUserUpdateAccess) -> Self {
        self.body.access = Some(value.into());
        self
    }

    #[inline]
    pub fn email(mut self, value: impl Into<String>) -> Self {
        self.body.email = Some(value.into());
        self
    }

    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> Self {
        self.body.enable = Some(value.into());
        self
    }

    #[inline]
    pub fn id(mut self, value: impl Into<String>) -> RequestUserUpdateBuilder<crate::generics::IdExists> {
        self.body.id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn locale(mut self, value: impl Into<String>) -> Self {
        self.body.locale = Some(value.into());
        self
    }

    #[inline]
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.body.name = Some(value.into());
        self
    }

    #[inline]
    pub fn password(mut self, value: crate::request_user_update::RequestUserUpdatePassword) -> Self {
        self.body.password = Some(value.into());
        self
    }

    #[inline]
    pub fn timezone(mut self, value: impl Into<String>) -> Self {
        self.body.timezone = Some(value.into());
        self
    }
}

/// Builder created by [`RequestUserUpdate::post`](./struct.RequestUserUpdate.html#method.post) method for a `POST` operation associated with `RequestUserUpdate`.
#[derive(Debug, Clone)]
pub struct RequestUserUpdatePostBuilder<Id> {
    body: self::RequestUserUpdate,
    _id: core::marker::PhantomData<Id>,
}

impl<Id> RequestUserUpdatePostBuilder<Id> {
    #[inline]
    pub fn access(mut self, value: crate::request_user_update::RequestUserUpdateAccess) -> Self {
        self.body.access = Some(value.into());
        self
    }

    #[inline]
    pub fn email(mut self, value: impl Into<String>) -> Self {
        self.body.email = Some(value.into());
        self
    }

    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> Self {
        self.body.enable = Some(value.into());
        self
    }

    #[inline]
    pub fn id(mut self, value: impl Into<String>) -> RequestUserUpdatePostBuilder<crate::generics::IdExists> {
        self.body.id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn locale(mut self, value: impl Into<String>) -> Self {
        self.body.locale = Some(value.into());
        self
    }

    #[inline]
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.body.name = Some(value.into());
        self
    }

    #[inline]
    pub fn password(mut self, value: crate::request_user_update::RequestUserUpdatePassword) -> Self {
        self.body.password = Some(value.into());
        self
    }

    #[inline]
    pub fn timezone(mut self, value: impl Into<String>) -> Self {
        self.body.timezone = Some(value.into());
        self
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestUserUpdatePostBuilder<crate::generics::IdExists> {
    type Output = crate::response_user::ResponseUser;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/user/update".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body))
    }
}

impl RequestUserUpdateAccess {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestUserUpdateAccessBuilder {
        RequestUserUpdateAccessBuilder {
            body: Default::default(),
        }
    }
}

impl Into<RequestUserUpdateAccess> for RequestUserUpdateAccessBuilder {
    fn into(self) -> RequestUserUpdateAccess {
        self.body
    }
}

/// Builder for [`RequestUserUpdateAccess`](./struct.RequestUserUpdateAccess.html) object.
#[derive(Debug, Clone)]
pub struct RequestUserUpdateAccessBuilder {
    body: self::RequestUserUpdateAccess,
}

impl RequestUserUpdateAccessBuilder {
    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> Self {
        self.body.enable = Some(value.into());
        self
    }

    #[inline]
    pub fn scope(mut self, value: impl Into<String>) -> Self {
        self.body.scope = Some(value.into());
        self
    }
}

impl RequestUserUpdatePassword {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestUserUpdatePasswordBuilder {
        RequestUserUpdatePasswordBuilder {
            body: Default::default(),
        }
    }
}

impl Into<RequestUserUpdatePassword> for RequestUserUpdatePasswordBuilder {
    fn into(self) -> RequestUserUpdatePassword {
        self.body
    }
}

/// Builder for [`RequestUserUpdatePassword`](./struct.RequestUserUpdatePassword.html) object.
#[derive(Debug, Clone)]
pub struct RequestUserUpdatePasswordBuilder {
    body: self::RequestUserUpdatePassword,
}

impl RequestUserUpdatePasswordBuilder {
    #[inline]
    pub fn allow_reset(mut self, value: impl Into<bool>) -> Self {
        self.body.allow_reset = Some(value.into());
        self
    }

    #[inline]
    pub fn require_update(mut self, value: impl Into<bool>) -> Self {
        self.body.require_update = Some(value.into());
        self
    }
}
