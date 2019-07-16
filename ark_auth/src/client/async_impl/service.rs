use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::api::{
    route, ServiceCreateBody, ServiceListQuery, ServiceListResponse, ServiceReadResponse,
};
use futures::Future;

impl AsyncClient {
    pub fn service_list(
        &self,
        gt: Option<&str>,
        lt: Option<&str>,
        limit: Option<i64>,
    ) -> impl Future<Item = ServiceListResponse, Error = Error> {
        let query = ServiceListQuery {
            gt: gt.map(|x| x.to_owned()),
            lt: lt.map(|x| x.to_owned()),
            limit: limit.map(|x| format!("{}", x)),
        };
        self.get_query(route::SERVICE, query)
            .send()
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<ServiceListResponse>().map_err(Into::into))
    }

    pub fn service_create(
        &self,
        is_enabled: bool,
        name: &str,
        url: &str,
    ) -> impl Future<Item = ServiceReadResponse, Error = Error> {
        let body = ServiceCreateBody {
            is_enabled,
            name: name.to_owned(),
            url: url.to_owned(),
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
}
