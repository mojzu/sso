#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestAuditRead {
    pub audit_type: Option<Vec<String>>,
    pub id: Option<Vec<i64>>,
    pub seek: crate::request_audit_read::RequestAuditReadSeek,
    pub subject: Option<Vec<String>>,
    pub user_id: Option<Vec<String>>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestAuditReadSeek {
    pub id: Option<i64>,
    pub limit: i64,
}

impl RequestAuditRead {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestAuditReadBuilder<crate::generics::MissingSeek> {
        RequestAuditReadBuilder {
            body: Default::default(),
            _seek: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn post() -> RequestAuditReadPostBuilder<crate::generics::MissingSeek> {
        RequestAuditReadPostBuilder {
            body: Default::default(),
            _seek: core::marker::PhantomData,
        }
    }
}

impl Into<RequestAuditRead> for RequestAuditReadBuilder<crate::generics::SeekExists> {
    fn into(self) -> RequestAuditRead {
        self.body
    }
}

impl Into<RequestAuditRead> for RequestAuditReadPostBuilder<crate::generics::SeekExists> {
    fn into(self) -> RequestAuditRead {
        self.body
    }
}

/// Builder for [`RequestAuditRead`](./struct.RequestAuditRead.html) object.
#[derive(Debug, Clone)]
pub struct RequestAuditReadBuilder<Seek> {
    body: self::RequestAuditRead,
    _seek: core::marker::PhantomData<Seek>,
}

impl<Seek> RequestAuditReadBuilder<Seek> {
    #[inline]
    pub fn audit_type(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.audit_type = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }

    #[inline]
    pub fn id(mut self, value: impl Iterator<Item = impl Into<i64>>) -> Self {
        self.body.id = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }

    #[inline]
    pub fn seek(mut self, value: crate::request_audit_read::RequestAuditReadSeekBuilder<crate::generics::LimitExists>) -> RequestAuditReadBuilder<crate::generics::SeekExists> {
        self.body.seek = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn subject(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.subject = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }

    #[inline]
    pub fn user_id(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.user_id = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }
}

/// Builder created by [`RequestAuditRead::post`](./struct.RequestAuditRead.html#method.post) method for a `POST` operation associated with `RequestAuditRead`.
#[derive(Debug, Clone)]
pub struct RequestAuditReadPostBuilder<Seek> {
    body: self::RequestAuditRead,
    _seek: core::marker::PhantomData<Seek>,
}

impl<Seek> RequestAuditReadPostBuilder<Seek> {
    #[inline]
    pub fn audit_type(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.audit_type = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }

    #[inline]
    pub fn id(mut self, value: impl Iterator<Item = impl Into<i64>>) -> Self {
        self.body.id = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }

    #[inline]
    pub fn seek(mut self, value: crate::request_audit_read::RequestAuditReadSeekBuilder<crate::generics::LimitExists>) -> RequestAuditReadPostBuilder<crate::generics::SeekExists> {
        self.body.seek = value.into();
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn subject(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.subject = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }

    #[inline]
    pub fn user_id(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.user_id = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestAuditReadPostBuilder<crate::generics::SeekExists> {
    type Output = crate::response_audit_many::ResponseAuditMany;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/audit/read".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body))
    }
}

impl RequestAuditReadSeek {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestAuditReadSeekBuilder<crate::generics::MissingLimit> {
        RequestAuditReadSeekBuilder {
            body: Default::default(),
            _limit: core::marker::PhantomData,
        }
    }
}

impl Into<RequestAuditReadSeek> for RequestAuditReadSeekBuilder<crate::generics::LimitExists> {
    fn into(self) -> RequestAuditReadSeek {
        self.body
    }
}

/// Builder for [`RequestAuditReadSeek`](./struct.RequestAuditReadSeek.html) object.
#[derive(Debug, Clone)]
pub struct RequestAuditReadSeekBuilder<Limit> {
    body: self::RequestAuditReadSeek,
    _limit: core::marker::PhantomData<Limit>,
}

impl<Limit> RequestAuditReadSeekBuilder<Limit> {
    #[inline]
    pub fn id(mut self, value: impl Into<i64>) -> Self {
        self.body.id = Some(value.into());
        self
    }

    #[inline]
    pub fn limit(mut self, value: impl Into<i64>) -> RequestAuditReadSeekBuilder<crate::generics::LimitExists> {
        self.body.limit = value.into();
        unsafe { std::mem::transmute(self) }
    }
}
