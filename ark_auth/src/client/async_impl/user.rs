use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::api::{route, UserCreateBody, UserCreateResponse, UserReadResponse};
use futures::Future;

impl AsyncClient {
    pub fn user_create<T: Into<String>>(
        &self,
        is_enabled: bool,
        name: T,
        email: T,
        password: Option<String>,
    ) -> impl Future<Item = UserCreateResponse, Error = Error> {
        let body = UserCreateBody {
            is_enabled,
            name: name.into(),
            email: email.into(),
            password,
        };

        self.post(route::USER)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<UserCreateResponse>().map_err(Into::into))
    }

    pub fn user_read(&self, id: &str) -> impl Future<Item = UserReadResponse, Error = Error> {
        let path = route::user_id(id);
        self.get(&path)
            .send()
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<UserReadResponse>().map_err(Into::into))
    }
}
