use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, FutureResult};
use futures::{Future, Poll};
use prometheus::{HistogramTimer, HistogramVec, IntCounterVec};

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
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
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
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        // TODO(refactor): Add path as label value (&[req.path()]), doesn't work now as
        // routing happens before middleware calls, so path contains ID, queries, etc.
        // Attaching this middleware to each route with a path could work, but verbose.
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
