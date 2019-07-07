use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::api::{
    ServiceCreateBody, ServiceListQuery, ServiceListResponse, ServiceReadResponse,
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

        self.get_query("/v1/service", query)
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

        self.post_json("/v1/service", &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<ServiceReadResponse>().map_err(Into::into))
    }

    pub fn service_read(&self, id: &str) -> Result<ServiceReadResponse, Error> {
        let path = format!("/v1/service/{}", id);

        self.get(&path)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<ServiceReadResponse>().map_err(Into::into))
    }
}
