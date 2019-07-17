use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::api::{
    route, AuditCreateBody, AuditListQuery, AuditListResponse, AuditReadResponse,
};

impl SyncClient {
    pub fn audit_list(&self, query: AuditListQuery) -> Result<AuditListResponse, Error> {
        self.get_query(route::AUDIT, query)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuditListResponse>().map_err(Into::into))
    }

    pub fn audit_create(&self, body: AuditCreateBody) -> Result<AuditReadResponse, Error> {
        self.post_json(route::AUDIT, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuditReadResponse>().map_err(Into::into))
    }

    pub fn audit_read(&self, id: &str) -> Result<AuditReadResponse, Error> {
        let path = route::audit_id(id);
        self.get(&path)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuditReadResponse>().map_err(Into::into))
    }
}
