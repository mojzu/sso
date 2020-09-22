#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseAudit {
    pub api_key_id: Option<String>,
    pub audit_type: String,
    pub client_id: Option<String>,
    pub created_at: String,
    pub data: Option<crate::response_audit::ResponseAuditData>,
    pub id: i64,
    pub status_code: Option<i32>,
    pub subject: Option<String>,
    pub token_id: Option<String>,
    pub user_id: Option<String>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseAuditData {}

impl ResponseAudit {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseAuditBuilder<crate::generics::MissingAuditType, crate::generics::MissingCreatedAt, crate::generics::MissingId> {
        ResponseAuditBuilder {
            body: Default::default(),
            _audit_type: core::marker::PhantomData,
            _created_at: core::marker::PhantomData,
            _id: core::marker::PhantomData,
        }
    }
}

impl Into<ResponseAudit> for ResponseAuditBuilder<crate::generics::AuditTypeExists, crate::generics::CreatedAtExists, crate::generics::IdExists> {
    fn into(self) -> ResponseAudit {
        self.body
    }
}

/// Builder for [`ResponseAudit`](./struct.ResponseAudit.html) object.
#[derive(Debug, Clone)]
pub struct ResponseAuditBuilder<AuditType, CreatedAt, Id> {
    body: self::ResponseAudit,
    _audit_type: core::marker::PhantomData<AuditType>,
    _created_at: core::marker::PhantomData<CreatedAt>,
    _id: core::marker::PhantomData<Id>,
}

impl<AuditType, CreatedAt, Id> ResponseAuditBuilder<AuditType, CreatedAt, Id> {
    #[inline]
    pub fn api_key_id(mut self, value: impl Into<String>) -> Self {
        self.body.api_key_id = Some(value.into());
        self
    }

    #[inline]
    pub fn audit_type(mut self, value: impl Into<String>) -> ResponseAuditBuilder<crate::generics::AuditTypeExists, CreatedAt, Id> {
        self.body.audit_type = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn client_id(mut self, value: impl Into<String>) -> Self {
        self.body.client_id = Some(value.into());
        self
    }

    #[inline]
    pub fn created_at(mut self, value: impl Into<String>) -> ResponseAuditBuilder<AuditType, crate::generics::CreatedAtExists, Id> {
        self.body.created_at = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn data(mut self, value: crate::response_audit::ResponseAuditData) -> Self {
        self.body.data = Some(value.into());
        self
    }

    #[inline]
    pub fn id(mut self, value: impl Into<i64>) -> ResponseAuditBuilder<AuditType, CreatedAt, crate::generics::IdExists> {
        self.body.id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn status_code(mut self, value: impl Into<i32>) -> Self {
        self.body.status_code = Some(value.into());
        self
    }

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

    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> Self {
        self.body.user_id = Some(value.into());
        self
    }
}

