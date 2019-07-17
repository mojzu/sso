use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::api::{
    route, KeyCreateBody, KeyListQuery, KeyListResponse, KeyReadResponse, KeyUpdateBody,
};
use futures::Future;

impl AsyncClient {
    pub fn key_list(
        &self,
        query: KeyListQuery,
    ) -> impl Future<Item = KeyListResponse, Error = Error> {
        self.get_query(route::KEY, query)
            .send()
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyListResponse>().map_err(Into::into))
    }

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

    pub fn key_read(&self, id: &str) -> impl Future<Item = KeyReadResponse, Error = Error> {
        let path = route::key_id(id);
        self.get(&path)
            .send()
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyReadResponse>().map_err(Into::into))
    }

    pub fn key_update(
        &self,
        id: &str,
        is_enabled: Option<bool>,
        name: Option<String>,
    ) -> impl Future<Item = KeyReadResponse, Error = Error> {
        let path = route::key_id(id);
        let body = KeyUpdateBody { is_enabled, name };
        self.patch(&path)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyReadResponse>().map_err(Into::into))
    }

    pub fn key_delete(&self, id: &str, forwarded: &str) -> impl Future<Item = (), Error = Error> {
        let path = route::key_id(id);
        self.delete(&path, forwarded)
            .send()
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }
}
