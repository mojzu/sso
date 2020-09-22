#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestAuditCreate {
    pub api_key_id: Option<String>,
    pub audit_type: String,
    /// Audit log data object
    pub data: crate::request_audit_create::RequestAuditCreateData,
    pub status_code: Option<i32>,
    /// Audit log subject
    pub subject: Option<String>,
    pub token_id: Option<String>,
    /// Audit log user UUID
    pub user_id: Option<String>,
}

/// Audit log data object
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestAuditCreateData {}

impl RequestAuditCreate {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestAuditCreateBuilder<crate::generics::MissingAuditType, crate::generics::MissingData> {
        RequestAuditCreateBuilder {
            body: Default::default(),
            _audit_type: core::marker::PhantomData,
            _data: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestAuditCreatePostBuilder<crate::generics::MissingAuditType, crate::generics::MissingData> {
        RequestAuditCreatePostBuilder {
            body: Default::default(),
            _audit_type: core::marker::PhantomData,
            _data: core::marker::PhantomData,
        }
    }
}

impl Into<RequestAuditCreate> for RequestAuditCreateBuilder<crate::generics::AuditTypeExists, crate::generics::DataExists> {
    fn into(self) -> RequestAuditCreate {
        self.body
    }
}

impl Into<RequestAuditCreate> for RequestAuditCreatePostBuilder<crate::generics::AuditTypeExists, crate::generics::DataExists> {
    fn into(self) -> RequestAuditCreate {
        self.body
    }
}

/// Builder for [`RequestAuditCreate`](./struct.RequestAuditCreate.html) object.
#[derive(Debug, Clone)]
pub struct RequestAuditCreateBuilder<AuditType, Data> {
    body: self::RequestAuditCreate,
    _audit_type: core::marker::PhantomData<AuditType>,
    _data: core::marker::PhantomData<Data>,
}

impl<AuditType, Data> RequestAuditCreateBuilder<AuditType, Data> {
    #[inline]
    pub fn api_key_id(mut self, value: impl Into<String>) -> Self {
        self.body.api_key_id = Some(value.into());
        self
    }

    #[inline]
    pub fn audit_type(mut self, value: impl Into<String>) -> RequestAuditCreateBuilder<crate::generics::AuditTypeExists, Data> {
        self.body.audit_type = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Audit log data object
    #[inline]
    pub fn data(mut self, value: crate::request_audit_create::RequestAuditCreateData) -> RequestAuditCreateBuilder<AuditType, crate::generics::DataExists> {
        self.body.data = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn status_code(mut self, value: impl Into<i32>) -> Self {
        self.body.status_code = Some(value.into());
        self
    }

    /// Audit log subject
    #[inline]
    pub fn subject(mut self, value: impl Into<String>) -> Self {
        self.body.subject = Some(value.into());
        self
    }

    #[inline]
    pub fn token_id(mut self, value: impl Into<String>) -> Self {
        self.body.token_id = Some(value.into());
        self
    }

    /// Audit log user UUID
    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> Self {
        self.body.user_id = Some(value.into());
        self
    }
}

/// Builder created by [`RequestAuditCreate::post`](./struct.RequestAuditCreate.html#method.post) method for a `POST` operation associated with `RequestAuditCreate`.
#[derive(Debug, Clone)]
pub struct RequestAuditCreatePostBuilder<AuditType, Data> {
    body: self::RequestAuditCreate,
    _audit_type: core::marker::PhantomData<AuditType>,
    _data: core::marker::PhantomData<Data>,
}

impl<AuditType, Data> RequestAuditCreatePostBuilder<AuditType, Data> {
    #[inline]
    pub fn api_key_id(mut self, value: impl Into<String>) -> Self {
        self.body.api_key_id = Some(value.into());
        self
    }

    #[inline]
    pub fn audit_type(mut self, value: impl Into<String>) -> RequestAuditCreatePostBuilder<crate::generics::AuditTypeExists, Data> {
        self.body.audit_type = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Audit log data object
    #[inline]
    pub fn data(mut self, value: crate::request_audit_create::RequestAuditCreateData) -> RequestAuditCreatePostBuilder<AuditType, crate::generics::DataExists> {
        self.body.data = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn status_code(mut self, value: impl Into<i32>) -> Self {
        self.body.status_code = Some(value.into());
        self
    }

    /// Audit log subject
    #[inline]
    pub fn subject(mut self, value: impl Into<String>) -> Self {
        self.body.subject = Some(value.into());
        self
    }

    #[inline]
    pub fn token_id(mut self, value: impl Into<String>) -> Self {
        self.body.token_id = Some(value.into());
        self
    }

    /// Audit log user UUID
    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> Self {
        self.body.user_id = Some(value.into());
        self
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestAuditCreatePostBuilder<crate::generics::AuditTypeExists, crate::generics::DataExists> {
    type Output = crate::response_audit::ResponseAudit;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/audit/create".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body))
    }
}

