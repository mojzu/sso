use crate::{Driver, DriverError, DriverResult, Service};
use chrono::{DateTime, Utc};
use prometheus::{
    Encoder, Gauge, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, Registry,
    TextEncoder,
};
use std::{convert::TryInto, fmt, sync::Mutex};
use sysinfo::{ProcessExt, System, SystemExt};

/// Metrics process CPU usage name.
pub const METRICS_PROCESS_CPU_USAGE_NAME: &str = "process_cpu_usage";

/// Metrics process CPU usage help.
pub const METRICS_PROCESS_CPU_USAGE_HELP: &str = "CPU usage (%)";

/// Metrics process resident memory name.
pub const METRICS_PROCESS_RESIDENT_MEMORY_NAME: &str = "process_resident_memory";

/// Metrics process resident memory help.
pub const METRICS_PROCESS_RESIDENT_MEMORY_HELP: &str = "Resident memory size (bytes)";

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
    pub process_cpu_usage: Gauge,
    pub process_resident_memory: IntGauge,
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
    static ref SYSTEM: Mutex<System> = { Mutex::new(System::new()) };
    static ref METRICS: Mutex<Metrics> = {
        let registry = Registry::new();

        let process_cpu_usage = Gauge::new(
            METRICS_PROCESS_CPU_USAGE_NAME,
            METRICS_PROCESS_CPU_USAGE_HELP,
        )
        .unwrap();

        let process_resident_memory = IntGauge::new(
            METRICS_PROCESS_RESIDENT_MEMORY_NAME,
            METRICS_PROCESS_RESIDENT_MEMORY_HELP,
        )
        .unwrap();

        let audit_count_opts = Opts::new(METRICS_AUDIT_COUNT_NAME, METRICS_AUDIT_COUNT_HELP);
        let audit_count = IntCounterVec::new(audit_count_opts, &["type", "status"]).unwrap();

        let grpc_count_opts = Opts::new(METRICS_GRPC_COUNT_NAME, METRICS_GRPC_COUNT_HELP);
        let grpc_count = IntCounterVec::new(grpc_count_opts, &["path", "status"]).unwrap();

        let grpc_latency_opts =
            HistogramOpts::new(METRICS_GRPC_LATENCY_NAME, METRICS_GRPC_LATENCY_NAME);
        let grpc_latency = HistogramVec::new(grpc_latency_opts, &["path"]).unwrap();

        registry
            .register(Box::new(process_cpu_usage.clone()))
            .unwrap();
        registry
            .register(Box::new(process_resident_memory.clone()))
            .unwrap();
        registry.register(Box::new(audit_count.clone())).unwrap();
        registry.register(Box::new(grpc_count.clone())).unwrap();
        registry.register(Box::new(grpc_latency.clone())).unwrap();

        Mutex::new(Metrics {
            registry,
            process_cpu_usage,
            process_resident_memory,
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

    pub fn read(driver: &dyn Driver, service: Option<&Service>) -> DriverResult<String> {
        let mut metrics = METRICS.lock().unwrap();
        let audit_metrics =
            driver.audit_read_metrics(&metrics.audit_from, service.map(|s| &s.id))?;

        metrics.audit_from = Utc::now();
        for (type_, status_code, count) in audit_metrics.iter() {
            let status_code = format!("{}", status_code);
            metrics
                .audit_count
                .with_label_values(&[type_, &status_code])
                .inc_by(*count);
        }

        Self::sysinfo(&metrics.process_cpu_usage, &metrics.process_resident_memory)?;
        Self::registry_encode(&metrics.registry)
    }

    pub fn sysinfo(
        process_cpu_usage: &Gauge,
        process_resident_memory: &IntGauge,
    ) -> DriverResult<()> {
        let mut system = SYSTEM.lock().unwrap();
        let pid = sysinfo::get_current_pid().unwrap();
        system.refresh_process(pid);
        let p = system.get_process(pid).unwrap();

        process_cpu_usage.set(f64::from(p.cpu_usage()));
        let memory_bytes: i64 = (p.memory() * 1024)
            .try_into()
            .map_err(|_e| DriverError::Metrics)?;
        process_resident_memory.set(memory_bytes);

        Ok(())
    }

    pub fn registry_encode(registry: &Registry) -> DriverResult<String> {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metrics = registry.gather();
        encoder.encode(&metrics, &mut buffer).unwrap();
        Ok(String::from_utf8(buffer).unwrap())
    }
}
