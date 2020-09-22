#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestUserRead {
    pub email: Option<Vec<String>>,
    pub id: Option<Vec<String>>,
}

impl RequestUserRead {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestUserReadBuilder {
        RequestUserReadBuilder {
            body: Default::default(),
        }
    }

    #[inline]
    pub fn post() -> RequestUserReadPostBuilder {
        RequestUserReadPostBuilder {
            body: Default::default(),
        }
    }
}

impl Into<RequestUserRead> for RequestUserReadBuilder {
    fn into(self) -> RequestUserRead {
        self.body
    }
}

impl Into<RequestUserRead> for RequestUserReadPostBuilder {
    fn into(self) -> RequestUserRead {
        self.body
    }
}

/// Builder for [`RequestUserRead`](./struct.RequestUserRead.html) object.
#[derive(Debug, Clone)]
pub struct RequestUserReadBuilder {
    body: self::RequestUserRead,
}

impl RequestUserReadBuilder {
    #[inline]
    pub fn email(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.email = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }

    #[inline]
    pub fn id(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.id = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }
}

/// Builder created by [`RequestUserRead::post`](./struct.RequestUserRead.html#method.post) method for a `POST` operation associated with `RequestUserRead`.
#[derive(Debug, Clone)]
pub struct RequestUserReadPostBuilder {
    body: self::RequestUserRead,
}

impl RequestUserReadPostBuilder {
    #[inline]
    pub fn email(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.email = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }

    #[inline]
    pub fn id(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.id = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestUserReadPostBuilder {
    type Output = crate::response_user_many::ResponseUserMany;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/user/read".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body))
    }
}
