use crate::{Driver, DriverResult};
use chrono::{DateTime, Utc};
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder,
};
use std::{fmt, sync::Mutex};

/// Metrics audit count name.
pub const METRICS_AUDIT_COUNT_NAME: &str = "audit_count";

/// Metrics audit count help.
pub const METRICS_AUDIT_COUNT_HELP: &str = "Audit log counter";

/// Metrics gRPC count name.
pub const METRICS_GRPC_COUNT_NAME: &str = "grpc_count";

/// Metrics gRPC count help.
pub const METRICS_GRPC_COUNT_HELP: &str = "gRPC request counter";

/// Metrics gRPC latency name.
pub const METRICS_GRPC_LATENCY_NAME: &str = "grpc_latency";

/// Metrics gRPC latency help.
pub const METRICS_GRPC_LATENCY_HELP: &str = "gRPC request latency (ms)";

/// Metrics.
pub struct Metrics {
    pub registry: Registry,
    pub audit_from: DateTime<Utc>,
    pub audit_count: IntCounterVec,
    pub grpc_count: IntCounterVec,
    pub grpc_latency: HistogramVec,
}

impl fmt::Debug for Metrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Metrics {{ registry, ... }}")
    }
}

lazy_static! {
    static ref METRICS: Mutex<Metrics> = {
        let registry = Registry::new();

        let audit_count_opts = Opts::new(METRICS_AUDIT_COUNT_NAME, METRICS_AUDIT_COUNT_HELP);
        let audit_count = IntCounterVec::new(audit_count_opts, &["type", "status"]).unwrap();

        let grpc_count_opts = Opts::new(METRICS_GRPC_COUNT_NAME, METRICS_GRPC_COUNT_HELP);
        let grpc_count = IntCounterVec::new(grpc_count_opts, &["path", "status"]).unwrap();

        let grpc_latency_opts =
            HistogramOpts::new(METRICS_GRPC_LATENCY_NAME, METRICS_GRPC_LATENCY_NAME);
        let grpc_latency = HistogramVec::new(grpc_latency_opts, &["path"]).unwrap();

        registry.register(Box::new(audit_count.clone())).unwrap();
        registry.register(Box::new(grpc_count.clone())).unwrap();
        registry.register(Box::new(grpc_latency.clone())).unwrap();

        Mutex::new(Metrics {
            registry,
            audit_from: Utc::now(),
            audit_count,
            grpc_count,
            grpc_latency,
        })
    };
}

impl Metrics {
    pub fn grpc_metrics() -> (IntCounterVec, HistogramVec) {
        let metrics = METRICS.lock().unwrap();
        (metrics.grpc_count.clone(), metrics.grpc_latency.clone())
    }

    pub fn read(driver: &dyn Driver) -> DriverResult<String> {
        let mut metrics = METRICS.lock().unwrap();
        let audit_metrics = driver.audit_read_metrics(&metrics.audit_from, None)?;

        metrics.audit_from = Utc::now();
        for (type_, status_code, count) in audit_metrics.iter() {
            let status_code = format!("{}", status_code);
            metrics
                .audit_count
                .with_label_values(&[type_, &status_code])
                .inc_by(*count);
        }

        Self::registry_encode(&metrics.registry)
    }

    pub fn registry_encode(registry: &Registry) -> DriverResult<String> {
        let encoder = TextEncoder::new();

        let mut buffer = vec![];
        let process_metrics = prometheus::gather();
        encoder.encode(&process_metrics, &mut buffer).unwrap();
        let process_metrics = String::from_utf8(buffer).unwrap();

        let mut buffer = vec![];
        let metrics = registry.gather();
        encoder.encode(&metrics, &mut buffer).unwrap();
        let metrics = String::from_utf8(buffer).unwrap();

        Ok(format!("{}\n\n{}", process_metrics, metrics))
    }
}
