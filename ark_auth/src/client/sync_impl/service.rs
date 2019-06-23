use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::route::service::{
    CreateBody, CreateResponse, ListQuery, ListResponse, ReadResponse,
};

impl SyncClient {
    pub fn service_list(
        &self,
        gt: Option<&str>,
        lt: Option<&str>,
        limit: Option<i64>,
    ) -> Result<ListResponse, Error> {
        let query = ListQuery {
            gt: gt.map(|x| x.to_owned()),
            lt: lt.map(|x| x.to_owned()),
            limit,
        };

        self.get_query("/v1/service", query)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<ListResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn service_create(
        &self,
        is_enabled: bool,
        name: &str,
        url: &str,
    ) -> Result<CreateResponse, Error> {
        let body = CreateBody {
            is_enabled,
            name: name.to_owned(),
            url: url.to_owned(),
        };

        self.post_json("/v1/service", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<CreateResponse>().map_err(|_err| Error::Unwrap))
    }

    pub fn service_read(&self, id: &str) -> Result<ReadResponse, Error> {
        let path = format!("/v1/service/{}", id);

        self.get(&path)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<ReadResponse>().map_err(|_err| Error::Unwrap))
    }
}
