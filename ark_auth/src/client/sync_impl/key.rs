use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::route::key::{CreateBody, CreateResponse};

impl SyncClient {
    pub fn key_create(
        &self,
        name: &str,
        service_id: Option<i64>,
        user_id: Option<i64>,
    ) -> Result<CreateResponse, Error> {
        let body = CreateBody {
            name: name.to_owned(),
            service_id,
            user_id,
        };

        self.post_json("/v1/key", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<CreateResponse>().map_err(|_err| Error::Unwrap))
    }
}
