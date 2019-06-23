use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::route::service::{
    CreateBody, CreateResponse, ListQuery, ListResponse, ReadResponse,
};
use futures::Future;

impl AsyncClient {
    pub fn service_list(
        &self,
        gt: Option<&str>,
        lt: Option<&str>,
        limit: Option<i64>,
    ) -> impl Future<Item = ListResponse, Error = Error> {
        let query = ListQuery {
            gt: gt.map(|x| x.to_owned()),
            lt: lt.map(|x| x.to_owned()),
            limit,
        };

        self.get_query("/v1/service", query)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<ListResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn service_create(
        &self,
        is_enabled: bool,
        name: &str,
        url: &str,
    ) -> impl Future<Item = CreateResponse, Error = Error> {
        let body = CreateBody {
            is_enabled,
            name: name.to_owned(),
            url: url.to_owned(),
        };

        self.post("/v1/service")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<CreateResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn service_read(&self, id: &str) -> impl Future<Item = ReadResponse, Error = Error> {
        let path = format!("/v1/service/{}", id);

        self.get(&path)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<ReadResponse>().map_err(|_err| Error::Unwrap))
    }
}
