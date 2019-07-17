use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::api::{route, KeyCreateBody, KeyReadResponse};
use futures::Future;

impl AsyncClient {
    pub fn key_create<T: Into<String>>(
        &self,
        is_enabled: bool,
        name: T,
        service_id: Option<String>,
        user_id: Option<String>,
    ) -> impl Future<Item = KeyReadResponse, Error = Error> {
        let body = KeyCreateBody {
            is_enabled,
            name: name.into(),
            service_id,
            user_id,
        };
        self.post(route::KEY)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyReadResponse>().map_err(Into::into))
    }
}
