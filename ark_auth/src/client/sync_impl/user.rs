use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::route::user::{CreateBody, CreateResponse};
use actix_web::http::StatusCode;

impl SyncClient {
    pub fn user_create(
        &self,
        name: &str,
        email: &str,
        active: bool,
        password: Option<&str>,
    ) -> Result<CreateResponse, Error> {
        let body = CreateBody {
            name: name.to_owned(),
            email: email.to_owned(),
            active,
            password: password.map(String::from),
        };

        self.post_json("/v1/user", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => Ok(res),
                _ => Err(Error::Unwrap),
            })
            .and_then(|mut res| res.json::<CreateResponse>().map_err(|_err| Error::Unwrap))
    }
}
