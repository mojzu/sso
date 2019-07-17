use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::api::{
    route, ServiceCreateBody, ServiceListQuery, ServiceListResponse, ServiceReadResponse,
    ServiceUpdateBody,
};

impl SyncClient {
    pub fn service_list(&self, query: ServiceListQuery) -> Result<ServiceListResponse, Error> {
        self.get_query(route::SERVICE, query)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<ServiceListResponse>().map_err(Into::into))
    }

    pub fn service_create<T: Into<String>>(
        &self,
        is_enabled: bool,
        name: T,
        url: T,
    ) -> Result<ServiceReadResponse, Error> {
        let body = ServiceCreateBody {
            is_enabled,
            name: name.into(),
            url: url.into(),
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

    pub fn service_update(
        &self,
        id: &str,
        is_enabled: Option<bool>,
        name: Option<String>,
    ) -> Result<ServiceReadResponse, Error> {
        let path = route::service_id(id);
        let body = ServiceUpdateBody { is_enabled, name };
        self.patch_json(&path, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<ServiceReadResponse>().map_err(Into::into))
    }

    pub fn service_delete(&self, id: &str) -> Result<(), Error> {
        let path = route::service_id(id);
        self.delete(&path)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .map(|_res| ())
    }
}
