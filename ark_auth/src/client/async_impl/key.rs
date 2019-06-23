use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::route::key::{CreateBody, CreateResponse};
use futures::Future;

impl AsyncClient {
    pub fn key_create(
        &self,
        is_enabled: bool,
        name: &str,
        service_id: Option<&str>,
        user_id: Option<&str>,
    ) -> impl Future<Item = CreateResponse, Error = Error> {
        let body = CreateBody {
            is_enabled,
            name: name.to_owned(),
            service_id: service_id.map(|x| x.to_owned()),
            user_id: user_id.map(|x| x.to_owned()),
        };

        self.post("/v1/key")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<CreateResponse>().map_err(|_err| Error::Unwrap))
    }
}
