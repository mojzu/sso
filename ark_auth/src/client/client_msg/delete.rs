use crate::{Client, ClientActor, ClientActorRequest, ClientError};
use actix::prelude::*;

/// Asynchronous client DELETE request.
#[derive(Debug, Serialize, Deserialize)]
pub struct Delete {
    url: String,
    route: String,
    authorisation: Option<String>,
    forwarded: Option<String>,
}

impl Delete {
    /// Create new DELETE request.
    pub fn new<T1, T2>(url: T1, route: T2) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            url: url.into(),
            route: route.into(),
            authorisation: None,
            forwarded: None,
        }
    }
}

impl ClientActorRequest for Delete {
    fn authorisation<T1: Into<String>>(mut self, authorisation: T1) -> Self {
        self.authorisation = Some(authorisation.into());
        self
    }

    fn forwarded<T1: Into<String>>(mut self, forwarded: T1) -> Self {
        self.forwarded = Some(forwarded.into());
        self
    }
}

impl Message for Delete {
    type Result = Result<String, ClientError>;
}

impl Handler<Delete> for ClientActor {
    type Result = ResponseActFuture<Self, String, ClientError>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Context<Self>) -> Self::Result {
        let url = Client::url(&msg.url, &msg.route).unwrap();
        let req = self.client().delete(url);
        let req = Client::request_headers(req, msg.authorisation, msg.forwarded);

        let res = req
            .send()
            .map_err(Into::into)
            .and_then(|res| res.error_for_status().map_err(Into::into))
            .and_then(|mut res| res.text().map_err(Into::into));

        let wrapped = actix::fut::wrap_future(res);
        Box::new(wrapped)
    }
}
