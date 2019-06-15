use crate::client::async_impl::AsyncClient;
use crate::client::Error;
use crate::server::route::user::{CreateBody, CreateResponse};
use actix_web::http::StatusCode;
use futures::{future, Future};

impl AsyncClient {
    pub fn user_create(
        &self,
        name: &str,
        email: &str,
        active: bool,
        password: Option<&str>,
    ) -> impl Future<Item = CreateResponse, Error = Error> {
        let body = CreateBody {
            name: name.to_owned(),
            email: email.to_owned(),
            active,
            password: password.map(String::from),
        };

        self.post("/v1/user")
            .send_json(&body)
            .map_err(|_err| Error::Unwrap)
            .and_then(|res| match res.status() {
                StatusCode::OK => future::ok(res),
                _ => future::err(Error::Unwrap),
            })
            .and_then(|mut res| res.json::<CreateResponse>().map_err(|_err| Error::Unwrap))
    }
}
