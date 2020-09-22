#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseApiKeyMany {
    pub data: Vec<crate::response_api_key_many::ResponseApiKeyManyDataItem>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResponseApiKeyManyDataItem {
    pub client_id: String,
    pub created_at: String,
    pub enable: bool,
    pub id: String,
    pub name: String,
    pub scope: String,
    pub updated_at: String,
    pub user_id: String,
    pub value: Option<String>,
}

impl ResponseApiKeyMany {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseApiKeyManyBuilder<crate::generics::MissingData> {
        ResponseApiKeyManyBuilder {
            body: Default::default(),
            _data: core::marker::PhantomData,
        }
    }
}

impl Into<ResponseApiKeyMany> for ResponseApiKeyManyBuilder<crate::generics::DataExists> {
    fn into(self) -> ResponseApiKeyMany {
        self.body
    }
}

/// Builder for [`ResponseApiKeyMany`](./struct.ResponseApiKeyMany.html) object.
#[derive(Debug, Clone)]
pub struct ResponseApiKeyManyBuilder<Data> {
    body: self::ResponseApiKeyMany,
    _data: core::marker::PhantomData<Data>,
}

impl<Data> ResponseApiKeyManyBuilder<Data> {
    #[inline]
    pub fn data(mut self, value: impl Iterator<Item = crate::response_api_key_many::ResponseApiKeyManyDataItemBuilder<crate::generics::ClientIdExists, crate::generics::CreatedAtExists, crate::generics::EnableExists, crate::generics::IdExists, crate::generics::NameExists, crate::generics::ScopeExists, crate::generics::UpdatedAtExists, crate::generics::UserIdExists>>) -> ResponseApiKeyManyBuilder<crate::generics::DataExists> {
        self.body.data = value.map(|value| value.into()).collect::<Vec<_>>().into();
        unsafe { std::mem::transmute(self) }
    }
}

impl ResponseApiKeyManyDataItem {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> ResponseApiKeyManyDataItemBuilder<crate::generics::MissingClientId, crate::generics::MissingCreatedAt, crate::generics::MissingEnable, crate::generics::MissingId, crate::generics::MissingName, crate::generics::MissingScope, crate::generics::MissingUpdatedAt, crate::generics::MissingUserId> {
        ResponseApiKeyManyDataItemBuilder {
            body: Default::default(),
            _client_id: core::marker::PhantomData,
            _created_at: core::marker::PhantomData,
            _enable: core::marker::PhantomData,
            _id: core::marker::PhantomData,
            _name: core::marker::PhantomData,
            _scope: core::marker::PhantomData,
            _updated_at: core::marker::PhantomData,
            _user_id: core::marker::PhantomData,
        }
    }
}

impl Into<ResponseApiKeyManyDataItem> for ResponseApiKeyManyDataItemBuilder<crate::generics::ClientIdExists, crate::generics::CreatedAtExists, crate::generics::EnableExists, crate::generics::IdExists, crate::generics::NameExists, crate::generics::ScopeExists, crate::generics::UpdatedAtExists, crate::generics::UserIdExists> {
    fn into(self) -> ResponseApiKeyManyDataItem {
        self.body
    }
}

/// Builder for [`ResponseApiKeyManyDataItem`](./struct.ResponseApiKeyManyDataItem.html) object.
#[derive(Debug, Clone)]
pub struct ResponseApiKeyManyDataItemBuilder<ClientId, CreatedAt, Enable, Id, Name, Scope, UpdatedAt, UserId> {
    body: self::ResponseApiKeyManyDataItem,
    _client_id: core::marker::PhantomData<ClientId>,
    _created_at: core::marker::PhantomData<CreatedAt>,
    _enable: core::marker::PhantomData<Enable>,
    _id: core::marker::PhantomData<Id>,
    _name: core::marker::PhantomData<Name>,
    _scope: core::marker::PhantomData<Scope>,
    _updated_at: core::marker::PhantomData<UpdatedAt>,
    _user_id: core::marker::PhantomData<UserId>,
}

impl<ClientId, CreatedAt, Enable, Id, Name, Scope, UpdatedAt, UserId> ResponseApiKeyManyDataItemBuilder<ClientId, CreatedAt, Enable, Id, Name, Scope, UpdatedAt, UserId> {
    #[inline]
    pub fn client_id(mut self, value: impl Into<String>) -> ResponseApiKeyManyDataItemBuilder<crate::generics::ClientIdExists, CreatedAt, Enable, Id, Name, Scope, UpdatedAt, UserId> {
        self.body.client_id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn created_at(mut self, value: impl Into<String>) -> ResponseApiKeyManyDataItemBuilder<ClientId, crate::generics::CreatedAtExists, Enable, Id, Name, Scope, UpdatedAt, UserId> {
        self.body.created_at = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn enable(mut self, value: impl Into<bool>) -> ResponseApiKeyManyDataItemBuilder<ClientId, CreatedAt, crate::generics::EnableExists, Id, Name, Scope, UpdatedAt, UserId> {
        self.body.enable = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn id(mut self, value: impl Into<String>) -> ResponseApiKeyManyDataItemBuilder<ClientId, CreatedAt, Enable, crate::generics::IdExists, Name, Scope, UpdatedAt, UserId> {
        self.body.id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn name(mut self, value: impl Into<String>) -> ResponseApiKeyManyDataItemBuilder<ClientId, CreatedAt, Enable, Id, crate::generics::NameExists, Scope, UpdatedAt, UserId> {
        self.body.name = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn scope(mut self, value: impl Into<String>) -> ResponseApiKeyManyDataItemBuilder<ClientId, CreatedAt, Enable, Id, Name, crate::generics::ScopeExists, UpdatedAt, UserId> {
        self.body.scope = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn updated_at(mut self, value: impl Into<String>) -> ResponseApiKeyManyDataItemBuilder<ClientId, CreatedAt, Enable, Id, Name, Scope, crate::generics::UpdatedAtExists, UserId> {
        self.body.updated_at = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn user_id(mut self, value: impl Into<String>) -> ResponseApiKeyManyDataItemBuilder<ClientId, CreatedAt, Enable, Id, Name, Scope, UpdatedAt, crate::generics::UserIdExists> {
        self.body.user_id = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.body.value = Some(value.into());
        self
    }
}
