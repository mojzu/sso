#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseAuditMany {
    pub data: Vec<crate::response_audit_many::ResponseAuditManyDataItem>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseAuditManyDataItem {
    pub api_key_id: Option<String>,
    pub audit_type: String,
    pub client_id: Option<String>,
    pub created_at: String,
    /// Audit log data object
    pub data: Option<crate::response_audit_many::ResponseAuditManyDataItemData>,
    pub id: i64,
    pub status_code: Option<i32>,
    pub subject: Option<String>,
    pub token_id: Option<String>,
    pub user_id: Option<String>,
}

/// Audit log data object
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseAuditManyDataItemData {}

impl ResponseAuditMany {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseAuditManyBuilder<crate::generics::MissingData> {
        ResponseAuditManyBuilder {
            body: Default::default(),
            _data: core::marker::PhantomData,
        }
    }
}

impl Into<ResponseAuditMany> for ResponseAuditManyBuilder<crate::generics::DataExists> {
    fn into(self) -> ResponseAuditMany {
        self.body
    }
}

/// Builder for [`ResponseAuditMany`](./struct.ResponseAuditMany.html) object.
#[derive(Debug, Clone)]
pub struct ResponseAuditManyBuilder<Data> {
    body: self::ResponseAuditMany,
    _data: core::marker::PhantomData<Data>,
}

impl<Data> ResponseAuditManyBuilder<Data> {
    #[inline]
    pub fn data(mut self, value: impl Iterator<Item = crate::response_audit_many::ResponseAuditManyDataItemBuilder<crate::generics::AuditTypeExists, crate::generics::CreatedAtExists, crate::generics::IdExists>>) -> ResponseAuditManyBuilder<crate::generics::DataExists> {
        self.body.data = value.map(|value| value.into()).collect::<Vec<_>>().into();
        unsafe { std::mem::transmute(self) }
    }
}

impl ResponseAuditManyDataItem {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseAuditManyDataItemBuilder<crate::generics::MissingAuditType, crate::generics::MissingCreatedAt, crate::generics::MissingId> {
        ResponseAuditManyDataItemBuilder {
            body: Default::default(),
            _audit_type: core::marker::PhantomData,
            _created_at: core::marker::PhantomData,
            _id: core::marker::PhantomData,
        }
    }
}

impl Into<ResponseAuditManyDataItem> for ResponseAuditManyDataItemBuilder<crate::generics::AuditTypeExists, crate::generics::CreatedAtExists, crate::generics::IdExists> {
    fn into(self) -> ResponseAuditManyDataItem {
        self.body
    }
}

/// Builder for [`ResponseAuditManyDataItem`](./struct.ResponseAuditManyDataItem.html) object.
#[derive(Debug, Clone)]
pub struct ResponseAuditManyDataItemBuilder<AuditType, CreatedAt, Id> {
    body: self::ResponseAuditManyDataItem,
    _audit_type: core::marker::PhantomData<AuditType>,
    _created_at: core::marker::PhantomData<CreatedAt>,
    _id: core::marker::PhantomData<Id>,
}

impl<AuditType, CreatedAt, Id> ResponseAuditManyDataItemBuilder<AuditType, CreatedAt, Id> {
    #[inline]
    pub fn api_key_id(mut self, value: impl Into<String>) -> Self {
        self.body.api_key_id = Some(value.into());
        self
    }

    #[inline]
    pub fn audit_type(mut self, value: impl Into<String>) -> ResponseAuditManyDataItemBuilder<crate::generics::AuditTypeExists, CreatedAt, Id> {
        self.body.audit_type = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn client_id(mut self, value: impl Into<String>) -> Self {
        self.body.client_id = Some(value.into());
        self
    }

    #[inline]
    pub fn created_at(mut self, value: impl Into<String>) -> ResponseAuditManyDataItemBuilder<AuditType, crate::generics::CreatedAtExists, Id> {
        self.body.created_at = value.into();
        unsafe { std::mem::transmute(self) }
    }

    /// Audit log data object
    #[inline]
    pub fn data(mut self, value: crate::response_audit_many::ResponseAuditManyDataItemData) -> Self {
        self.body.data = Some(value.into());
        self
    }

    #[inline]
    pub fn id(mut self, value: impl Into<i64>) -> ResponseAuditManyDataItemBuilder<AuditType, CreatedAt, crate::generics::IdExists> {
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

