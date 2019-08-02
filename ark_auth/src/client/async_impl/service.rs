use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::api::{
    route, ServiceCreateBody, ServiceListQuery, ServiceListResponse, ServiceReadResponse,
    ServiceUpdateBody,
};
use futures::Future;

impl AsyncClient {
    pub fn service_list(
        &self,
        query: ServiceListQuery,
    ) -> impl Future<Item = ServiceListResponse, Error = Error> {
        self.get_query(route::SERVICE, query)
            .send()
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<ServiceListResponse>().map_err(Into::into))
    }

    pub fn service_create<T: Into<String>>(
        &self,
        is_enabled: bool,
        name: T,
        url: T,
    ) -> impl Future<Item = ServiceReadResponse, Error = Error> {
        let body = ServiceCreateBody {
            is_enabled,
            name: name.into(),
            url: url.into(),
        };
        self.post(route::SERVICE)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<ServiceReadResponse>().map_err(Into::into))
    }

    pub fn service_read(&self, id: &str) -> impl Future<Item = ServiceReadResponse, Error = Error> {
        let path = route::service_id(id);
        self.get(&path)
            .send()
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<ServiceReadResponse>().map_err(Into::into))
    }

    pub fn service_update(
        &self,
        id: &str,
        is_enabled: Option<bool>,
        name: Option<String>,
    ) -> impl Future<Item = ServiceReadResponse, Error = Error> {
        let path = route::service_id(id);
        let body = ServiceUpdateBody { is_enabled, name };
        self.patch(&path)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<ServiceReadResponse>().map_err(Into::into))
    }

    pub fn service_delete(&self, id: &str) -> impl Future<Item = (), Error = Error> {
        let path = route::service_id(id);
        self.delete(&path)
            .send()
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .map(|_res| ())
    }
}
