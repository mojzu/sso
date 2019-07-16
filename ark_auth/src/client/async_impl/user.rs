use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::api::{route, UserCreateBody, UserCreateResponse};
use futures::Future;

impl AsyncClient {
    pub fn user_create(
        &self,
        is_enabled: bool,
        name: &str,
        email: &str,
        password: Option<&str>,
    ) -> impl Future<Item = UserCreateResponse, Error = Error> {
        let body = UserCreateBody {
            is_enabled,
            name: name.to_owned(),
            email: email.to_owned(),
            password: password.map(String::from),
        };

        self.post(route::USER)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<UserCreateResponse>().map_err(Into::into))
    }
}
