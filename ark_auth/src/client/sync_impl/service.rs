use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::api::{
    route, ServiceCreateBody, ServiceListQuery, ServiceListResponse, ServiceReadResponse,
};

impl SyncClient {
    pub fn service_list(
        &self,
        gt: Option<&str>,
        lt: Option<&str>,
        limit: Option<i64>,
    ) -> Result<ServiceListResponse, Error> {
        let query = ServiceListQuery {
            gt: gt.map(|x| x.to_owned()),
            lt: lt.map(|x| x.to_owned()),
            limit: limit.map(|x| format!("{}", x)),
        };
        self.get_query(route::SERVICE, query)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<ServiceListResponse>().map_err(Into::into))
    }

    pub fn service_create(
        &self,
        is_enabled: bool,
        name: &str,
        url: &str,
    ) -> Result<ServiceReadResponse, Error> {
        let body = ServiceCreateBody {
            is_enabled,
            name: name.to_owned(),
            url: url.to_owned(),
        };
        self.post_json(route::SERVICE, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<ServiceReadResponse>().map_err(Into::into))
    }

    pub fn service_read(&self, id: &str) -> Result<ServiceReadResponse, Error> {
        let path = route::service_id(id);
        self.get(&path)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<ServiceReadResponse>().map_err(Into::into))
    }
}
