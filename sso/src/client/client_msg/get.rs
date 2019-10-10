use crate::{Client, ClientActor, ClientActorRequest, ClientError, ClientResult, CoreUtil};
use actix::prelude::*;
use serde::ser::Serialize;

/// GET request message.
#[derive(Debug, Serialize, Deserialize)]
pub struct Get {
    url: String,
    route: String,
    query: Option<String>,
    // TODO(refactor): Headers HashMap.
    authorisation: Option<String>,
    forwarded: Option<String>,
}

impl Get {
    /// Create new GET request.
    pub fn new<T1, T2>(url: T1, route: T2) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            url: url.into(),
            route: route.into(),
            query: None,
            authorisation: None,
            forwarded: None,
        }
    }

    /// Set query string on GET request URL.
    pub fn query<S: Serialize>(mut self, query: S) -> ClientResult<Self> {
        let query = CoreUtil::qs_ser(&query).map_err(ClientError::core)?;
        self.query = Some(query);
        Ok(self)
    }
}

impl ClientActorRequest for Get {
    fn authorisation<T1: Into<String>>(mut self, authorisation: T1) -> Self {
        self.authorisation = Some(authorisation.into());
        self
    }

    fn forwarded<T1: Into<String>>(mut self, forwarded: T1) -> Self {
        self.forwarded = Some(forwarded.into());
        self
    }
}

impl Message for Get {
    type Result = Result<String, ClientError>;
}

impl Handler<Get> for ClientActor {
    type Result = ResponseActFuture<Self, String, ClientError>;

    fn handle(&mut self, msg: Get, _ctx: &mut Context<Self>) -> Self::Result {
        let mut url = Client::url(&msg.url, &msg.route).unwrap();
        url.set_query(msg.query.as_ref().map(|x| &**x));

        let req = self.client().get(url);
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
