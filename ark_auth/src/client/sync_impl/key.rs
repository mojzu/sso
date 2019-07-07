use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::api::{KeyCreateBody, KeyListQuery, KeyListResponse, KeyReadResponse};

impl SyncClient {
    pub fn key_list(
        &self,
        gt: Option<&str>,
        lt: Option<&str>,
        limit: Option<i64>,
    ) -> Result<KeyListResponse, Error> {
        let query = KeyListQuery {
            gt: gt.map(|x| x.to_owned()),
            lt: lt.map(|x| x.to_owned()),
            limit: limit.map(|x| format!("{}", x)),
        };

        self.get_query("/v1/key", query)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyListResponse>().map_err(Into::into))
    }

    pub fn key_create(
        &self,
        is_enabled: bool,
        name: &str,
        service_id: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<KeyReadResponse, Error> {
        let body = KeyCreateBody {
            is_enabled,
            name: name.to_owned(),
            service_id: service_id.map(|x| x.to_owned()),
            user_id: user_id.map(|x| x.to_owned()),
        };

        self.post_json("/v1/key", &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyReadResponse>().map_err(Into::into))
    }

    pub fn key_read(&self, id: &str) -> Result<KeyReadResponse, Error> {
        let path = format!("/v1/key/{}", id);

        self.get(&path)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<KeyReadResponse>().map_err(Into::into))
    }
}
