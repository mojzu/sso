use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::api::{
    AuditCreateBody, AuditListQuery, AuditListResponse, AuditReadResponse,
};
use serde_json::Value;

impl SyncClient {
    pub fn audit_list(
        &self,
        gt: Option<&str>,
        lt: Option<&str>,
        limit: Option<i64>,
    ) -> Result<AuditListResponse, Error> {
        let query = AuditListQuery {
            gt: gt.map(|x| x.to_owned()),
            lt: lt.map(|x| x.to_owned()),
            limit: limit.map(|x| format!("{}", x)),
        };

        self.get_query("/v1/audit", query)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| {
                res.json::<AuditListResponse>()
                    .map_err(|_err| Error::Unwrap)
            })
    }

    pub fn audit_create(
        &self,
        path: &str,
        data: &Value,
        user_id: Option<&str>,
        user_key_id: Option<&str>,
    ) -> Result<AuditReadResponse, Error> {
        let body = AuditCreateBody {
            path: path.to_owned(),
            data: data.to_owned(),
            user_id: user_id.map(|x| x.to_owned()),
            user_key_id: user_key_id.map(|x| x.to_owned()),
        };

        self.post_json("/v1/audit", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| {
                res.json::<AuditReadResponse>()
                    .map_err(|_err| Error::Unwrap)
            })
    }

    pub fn audit_read(&self, id: &str) -> Result<AuditReadResponse, Error> {
        let path = format!("/v1/audit/{}", id);

        self.get(&path)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| {
                res.json::<AuditReadResponse>()
                    .map_err(|_err| Error::Unwrap)
            })
    }
}
