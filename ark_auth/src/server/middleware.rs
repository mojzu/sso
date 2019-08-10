use crate::server::error::Error;
use actix_identity::{IdentityPolicy, IdentityService};
use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{Error as ActixWebError, Result as ActixWebResult};
use futures::future::{ok, FutureResult};
use futures::{Future, Poll};
use prometheus::{HistogramTimer, HistogramVec, IntCounterVec};

/// Authorisation identity policy middleware.
pub struct AuthorisationIdentityPolicy {
    header: String,
}

impl AuthorisationIdentityPolicy {
    /// Create new identity service.
    pub fn identity_service() -> IdentityService<Self> {
        IdentityService::new(AuthorisationIdentityPolicy::default())
    }
}

impl Default for AuthorisationIdentityPolicy {
    fn default() -> Self {
        AuthorisationIdentityPolicy {
            header: "Authorization".to_owned(),
        }
    }
}

impl IdentityPolicy for AuthorisationIdentityPolicy {
    type Future = ActixWebResult<Option<String>, ActixWebError>;
    type ResponseFuture = ActixWebResult<(), ActixWebError>;

    fn from_request(&self, request: &mut ServiceRequest) -> Self::Future {
        let key = match request.headers().get(&self.header) {
            Some(value) => {
                let value = value.to_str().map_err(|_err| Error::Forbidden)?;
                trim_authorisation(value)
            }
            None => None,
        };
        Ok(key)
    }

    fn to_response<B>(
        &self,
        _id: Option<String>,
        _changed: bool,
        _response: &mut ServiceResponse<B>,
    ) -> Self::ResponseFuture {
        Ok(())
    }
}

/// Returns key value from formats: `$KEY`, `Bearer $KEY`.
fn trim_authorisation(value: &str) -> Option<String> {
    let value = value.to_owned();
    if value.starts_with("Bearer ") {
        let parts: Vec<&str> = value.split(' ').collect();
        if parts.len() > 1 {
            let value = parts[1].trim().to_owned();
            Some(value)
        } else {
            None
        }
    } else {
        Some(value)
    }
}

/// Metrics middleware.
pub struct Metrics {
    count: IntCounterVec,
    latency: HistogramVec,
}

impl Metrics {
    pub fn new(count: IntCounterVec, latency: HistogramVec) -> Self {
        Self { count, latency }
    }
}

impl<S, B> Transform<S> for Metrics
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = ActixWebError>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = ActixWebError;
    type InitError = ();
    type Transform = MetricsMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MetricsMiddleware {
            service,
            count: self.count.clone(),
            latency: self.latency.clone(),
        })
    }
}

pub struct MetricsMiddleware<S> {
    service: S,
    count: IntCounterVec,
    latency: HistogramVec,
}

impl<S, B> Service for MetricsMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = ActixWebError>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = ActixWebError;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        // TODO(refactor): Add path as label value (&[req.path()]).
        // <https://github.com/actix/actix-web/issues/833>
        let timer = self.latency.with_label_values(&["/"]).start_timer();
        let timer = ok::<HistogramTimer, Self::Error>(timer);
        let count = self.count.clone();

        Box::new(
            self.service
                .call(req)
                .join(timer)
                .and_then(move |(res, timer)| {
                    timer.observe_duration();
                    count
                        .with_label_values(&["/", res.status().as_str()])
                        .inc_by(1);
                    Ok(res)
                }),
        )
    }
}
