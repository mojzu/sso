use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::api::{
    route, KeyCreateBody, KeyListQuery, KeyListResponse, KeyReadResponse, KeyUpdateBody,
};

impl SyncClient {
    pub fn key_list(&self, query: KeyListQuery) -> Result<KeyListResponse, Error> {
        self.get_query(route::KEY, query)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyListResponse>().map_err(Into::into))
    }

    pub fn key_create<T: Into<String>>(
        &self,
        is_enabled: bool,
        name: T,
        service_id: Option<String>,
        user_id: Option<String>,
    ) -> Result<KeyReadResponse, Error> {
        let body = KeyCreateBody {
            is_enabled,
            name: name.into(),
            service_id,
            user_id,
        };
        self.post_json(route::KEY, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyReadResponse>().map_err(Into::into))
    }

    pub fn key_read(&self, id: &str) -> Result<KeyReadResponse, Error> {
        let path = route::key_id(id);
        self.get(&path)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyReadResponse>().map_err(Into::into))
    }

    pub fn key_update(
        &self,
        id: &str,
        is_enabled: Option<bool>,
        name: Option<String>,
    ) -> Result<KeyReadResponse, Error> {
        let path = route::key_id(id);
        let body = KeyUpdateBody { is_enabled, name };
        self.patch_json(&path, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyReadResponse>().map_err(Into::into))
    }

    pub fn key_delete(&self, id: &str) -> Result<(), Error> {
        let path = route::key_id(id);
        self.delete(&path)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .map(|_res| ())
    }
}
