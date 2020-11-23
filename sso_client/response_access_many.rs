#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseAccessMany {
    pub data: Vec<crate::response_access_many::ResponseAccessManyDataItem>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseAccessManyDataItem {
    pub client_id: String,
    pub created_at: String,
    pub enable: bool,
    pub scope: String,
    #[serde(rename = "static")]
    pub static_: bool,
    pub updated_at: String,
    pub user_id: String,
}

impl ResponseAccessMany {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseAccessManyBuilder<crate::generics::MissingData> {
        ResponseAccessManyBuilder {
            body: Default::default(),
            _data: core::marker::PhantomData,
        }
    }
}

impl Into<ResponseAccessMany> for ResponseAccessManyBuilder<crate::generics::DataExists> {
    fn into(self) -> ResponseAccessMany {
        self.body
    }
}

/// Builder for [`ResponseAccessMany`](./struct.ResponseAccessMany.html) object.
#[derive(Debug, Clone)]
pub struct ResponseAccessManyBuilder<Data> {
    body: self::ResponseAccessMany,
    _data: core::marker::PhantomData<Data>,
}

impl<Data> ResponseAccessManyBuilder<Data> {
    #[inline]
    pub fn data(mut self, value: impl Iterator<Item = crate::response_access_many::ResponseAccessManyDataItemBuilder<crate::generics::ClientIdExists, crate::generics::CreatedAtExists, crate::generics::EnableExists, crate::generics::ScopeExists, crate::generics::StaticExists, crate::generics::UpdatedAtExists, crate::generics::UserIdExists>>) -> ResponseAccessManyBuilder<crate::generics::DataExists> {
        self.body.data = value.map(|value| value.into()).collect::<Vec<_>>().into();
        unsafe { std::mem::transmute(self) }
    }
}

impl ResponseAccessManyDataItem {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseAccessManyDataItemBuilder<crate::generics::MissingClientId, crate::generics::MissingCreatedAt, crate::generics::MissingEnable, crate::generics::MissingScope, crate::generics::MissingStatic, crate::generics::MissingUpdatedAt, crate::generics::MissingUserId> {
        ResponseAccessManyDataItemBuilder {
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

impl Into<ResponseAccessManyDataItem> for ResponseAccessManyDataItemBuilder<crate::generics::ClientIdExists, crate::generics::CreatedAtExists, crate::generics::EnableExists, crate::generics::ScopeExists, crate::generics::StaticExists, crate::generics::UpdatedAtExists, crate::generics::UserIdExists> {
    fn into(self) -> ResponseAccessManyDataItem {
        self.body
    }
}

/// Builder for [`ResponseAccessManyDataItem`](./struct.ResponseAccessManyDataItem.html) object.
#[derive(Debug, Clone)]
pub struct ResponseAccessManyDataItemBuilder<ClientId, CreatedAt, Enable, Scope, Static, UpdatedAt, UserId> {
    body: self::ResponseAccessManyDataItem,
    _client_id: core::marker::PhantomData<ClientId>,
    _created_at: core::marker::PhantomData<CreatedAt>,
    _enable: core::marker::PhantomData<Enable>,
    _scope: core::marker::PhantomData<Scope>,
    _static: core::marker::PhantomData<Static>,
    _updated_at: core::marker::PhantomData<UpdatedAt>,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<ClientId, CreatedAt, Enable, Scope, Static, UpdatedAt, UserId> ResponseAccessManyDataItemBuilder<ClientId, CreatedAt, Enable, Scope, Static, UpdatedAt, UserId> {
    #[inline]
    pub fn client_id(mut self, value: impl Into<String>) -> ResponseAccessManyDataItemBuilder<crate::generics::ClientIdExists, CreatedAt, Enable, Scope, Static, UpdatedAt, UserId> {
        self.body.client_id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn created_at(mut self, value: impl Into<String>) -> ResponseAccessManyDataItemBuilder<ClientId, crate::generics::CreatedAtExists, Enable, Scope, Static, UpdatedAt, UserId> {
        self.body.created_at = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> ResponseAccessManyDataItemBuilder<ClientId, CreatedAt, crate::generics::EnableExists, Scope, Static, UpdatedAt, UserId> {
        self.body.enable = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn scope(mut self, value: impl Into<String>) -> ResponseAccessManyDataItemBuilder<ClientId, CreatedAt, Enable, crate::generics::ScopeExists, Static, UpdatedAt, UserId> {
        self.body.scope = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn static_(mut self, value: impl Into<bool>) -> ResponseAccessManyDataItemBuilder<ClientId, CreatedAt, Enable, Scope, crate::generics::StaticExists, UpdatedAt, UserId> {
        self.body.static_ = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn updated_at(mut self, value: impl Into<String>) -> ResponseAccessManyDataItemBuilder<ClientId, CreatedAt, Enable, Scope, Static, crate::generics::UpdatedAtExists, UserId> {
        self.body.updated_at = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> ResponseAccessManyDataItemBuilder<ClientId, CreatedAt, Enable, Scope, Static, UpdatedAt, crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }
}
