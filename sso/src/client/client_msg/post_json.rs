use crate::{Client, ClientActor, ClientActorRequest, ClientError};
use actix::prelude::*;
use serde::ser::Serialize;

/// POST JSON request message.
#[derive(Debug)]
pub struct PostJson<S: Serialize> {
    url: String,
    route: String,
    authorisation: Option<String>,
    forwarded: Option<String>,
    body: Option<S>,
}

impl<S: Serialize> PostJson<S> {
    /// Create new POST JSON request.
    pub fn new<T1, T2>(url: T1, route: T2, body: Option<S>) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            url: url.into(),
            route: route.into(),
            authorisation: None,
            forwarded: None,
            body,
        }
    }
}

impl<S: Serialize> ClientActorRequest for PostJson<S> {
    fn authorisation<T1: Into<String>>(mut self, authorisation: T1) -> Self {
        self.authorisation = Some(authorisation.into());
        self
    }

    fn forwarded<T1: Into<String>>(mut self, forwarded: T1) -> Self {
        self.forwarded = Some(forwarded.into());
        self
    }
}

impl<S: Serialize> Message for PostJson<S> {
    type Result = Result<String, ClientError>;
}

impl<S: Serialize> Handler<PostJson<S>> for ClientActor {
    type Result = ResponseActFuture<Self, String, ClientError>;

    fn handle(&mut self, msg: PostJson<S>, _ctx: &mut Context<Self>) -> Self::Result {
        let url = Client::url(&msg.url, &msg.route).unwrap();
        let req = self.client().post(url);
        let req = Client::request_headers(req, msg.authorisation, msg.forwarded);
        let req = if let Some(body) = msg.body {
            req.json(&body)
        } else {
            req
        };

        let res = req
            .send()
            .map_err(Into::into)
            .and_then(|res| res.error_for_status().map_err(Into::into))
            .and_then(|mut res| res.text().map_err(Into::into));

        let wrapped = actix::fut::wrap_future(res);
        Box::new(wrapped)
    }
}
