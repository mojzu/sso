use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::route::key::{CreateBody, CreateResponse};

impl SyncClient {
    pub fn key_create(
        &self,
        is_active: bool,
        name: &str,
        service_id: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<CreateResponse, Error> {
        let body = CreateBody {
            is_active,
            name: name.to_owned(),
            service_id: service_id.map(|x| x.to_owned()),
            user_id: user_id.map(|x| x.to_owned()),
        };

        self.post_json("/v1/key", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<CreateResponse>().map_err(|_err| Error::Unwrap))
    }
}
