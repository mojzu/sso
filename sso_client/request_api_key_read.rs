#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestApiKeyRead {
    pub id: Option<Vec<String>>,
    pub user_id: Option<Vec<String>>,
}

impl RequestApiKeyRead {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> RequestApiKeyReadBuilder {
        RequestApiKeyReadBuilder {
            body: Default::default(),
        }
    }

    #[inline]
    pub fn post() -> RequestApiKeyReadPostBuilder {
        RequestApiKeyReadPostBuilder {
            body: Default::default(),
        }
    }
}

impl Into<RequestApiKeyRead> for RequestApiKeyReadBuilder {
    fn into(self) -> RequestApiKeyRead {
        self.body
    }
}

impl Into<RequestApiKeyRead> for RequestApiKeyReadPostBuilder {
    fn into(self) -> RequestApiKeyRead {
        self.body
    }
}

/// Builder for [`RequestApiKeyRead`](./struct.RequestApiKeyRead.html) object.
#[derive(Debug, Clone)]
pub struct RequestApiKeyReadBuilder {
    body: self::RequestApiKeyRead,
}

impl RequestApiKeyReadBuilder {
    #[inline]
    pub fn id(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.id = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }

    #[inline]
    pub fn user_id(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.user_id = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }
}

/// Builder created by [`RequestApiKeyRead::post`](./struct.RequestApiKeyRead.html#method.post) method for a `POST` operation associated with `RequestApiKeyRead`.
#[derive(Debug, Clone)]
pub struct RequestApiKeyReadPostBuilder {
    body: self::RequestApiKeyRead,
}

impl RequestApiKeyReadPostBuilder {
    #[inline]
    pub fn id(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.id = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }

    #[inline]
    pub fn user_id(mut self, value: impl Iterator<Item = impl Into<String>>) -> Self {
        self.body.user_id = Some(value.map(|value| value.into()).collect::<Vec<_>>().into());
        self
    }
}

impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestApiKeyReadPostBuilder {
    type Output = crate::response_api_key_many::ResponseApiKeyMany;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/user/api-key/read".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body))
    }
}
