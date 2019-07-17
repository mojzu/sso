use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::api::{
    route, AuditCreateBody, AuditListQuery, AuditListResponse, AuditReadResponse,
};
use futures::Future;

impl AsyncClient {
    pub fn audit_list(
        &self,
        query: AuditListQuery,
    ) -> impl Future<Item = AuditListResponse, Error = Error> {
        self.get_query(route::AUDIT, query)
            .send()
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuditListResponse>().map_err(Into::into))
    }

    pub fn audit_create(
        &self,
        body: AuditCreateBody,
    ) -> impl Future<Item = AuditReadResponse, Error = Error> {
        self.post(route::AUDIT)
            .send_json(&body)
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuditReadResponse>().map_err(Into::into))
    }

    pub fn audit_read(&self, id: &str) -> impl Future<Item = AuditReadResponse, Error = Error> {
        let path = route::audit_id(id);
        self.get(&path)
            .send()
            .map_err(Into::into)
            .and_then(AsyncClient::match_status_code)
            .and_then(|mut res| res.json::<AuditReadResponse>().map_err(Into::into))
    }
}
