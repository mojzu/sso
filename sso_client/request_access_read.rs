#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RequestAccessRead {}

impl RequestAccessRead {
    #[inline]
    pub fn post() -> RequestAccessReadPostBuilder {
        RequestAccessReadPostBuilder {
            body: Default::default(),
        }
    }
}

impl Into<RequestAccessRead> for RequestAccessReadPostBuilder {
    fn into(self) -> RequestAccessRead {
        self.body
    }
}

/// Builder created by [`RequestAccessRead::post`](./struct.RequestAccessRead.html#method.post) method for a `POST` operation associated with `RequestAccessRead`.
#[derive(Debug, Clone)]
pub struct RequestAccessReadPostBuilder {
    body: self::RequestAccessRead,
}


impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for RequestAccessReadPostBuilder {
    type Output = crate::response_access_many::ResponseAccessMany;

    const METHOD: http::Method = http::Method::POST;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/v2/client/access/read".into()
    }

    fn modify(&self, req: Client::Request) -> Result<Client::Request, crate::client::ApiError<Client::Response>> {
        use crate::client::Request;
        Ok(req
        .json(&self.body))
    }
}
