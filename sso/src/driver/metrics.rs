use crate::{Driver, DriverError, DriverResult, Service};
use chrono::{DateTime, Utc};
use prometheus::{
    Counter, Encoder, HistogramOpts, HistogramVec, IntCounter, IntCounterVec, Opts, Registry,
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

/// Metrics HTTP count name.
pub const METRICS_HTTP_COUNT_NAME: &str = "http_count";

/// Metrics HTTP count help.
pub const METRICS_HTTP_COUNT_HELP: &str = "HTTP request counter";

/// Metrics HTTP latency name.
pub const METRICS_HTTP_LATENCY_NAME: &str = "http_latency";

/// Metrics HTTP latency help.
pub const METRICS_HTTP_LATENCY_HELP: &str = "HTTP request latency";

/// Metrics.
pub struct Metrics {
    pub registry: Registry,
    pub process_cpu_usage: Counter,
    pub process_resident_memory: IntCounter,
    pub audit_from: DateTime<Utc>,
    pub audit_count: IntCounterVec,
    pub http_count: IntCounterVec,
    pub http_latency: HistogramVec,
}

impl fmt::Debug for Metrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Metrics {{ registry, process_cpu_usage, process_resident_memory, audit_from, audit_count, http_count, http_latency }}"
        )
    }
}

lazy_static! {
    static ref SYSTEM: Mutex<System> = { Mutex::new(System::new()) };
    static ref METRICS: Mutex<Metrics> = {
        let registry = Registry::new();

        let process_cpu_usage = Counter::new(
            METRICS_PROCESS_CPU_USAGE_NAME,
            METRICS_PROCESS_CPU_USAGE_HELP,
        )
        .unwrap();

        let process_resident_memory = IntCounter::new(
            METRICS_PROCESS_RESIDENT_MEMORY_NAME,
            METRICS_PROCESS_RESIDENT_MEMORY_HELP,
        )
        .unwrap();

        let audit_count_opts = Opts::new(METRICS_AUDIT_COUNT_NAME, METRICS_AUDIT_COUNT_HELP);
        let audit_count = IntCounterVec::new(audit_count_opts, &["path"]).unwrap();

        let http_count_opts = Opts::new(METRICS_HTTP_COUNT_NAME, METRICS_HTTP_COUNT_HELP);
        let http_count = IntCounterVec::new(http_count_opts, &["path", "status"]).unwrap();

        let http_latency_opts =
            HistogramOpts::new(METRICS_HTTP_LATENCY_NAME, METRICS_HTTP_LATENCY_NAME);
        let http_latency = HistogramVec::new(http_latency_opts, &["path"]).unwrap();

        registry
            .register(Box::new(process_cpu_usage.clone()))
            .unwrap();
        registry
            .register(Box::new(process_resident_memory.clone()))
            .unwrap();
        registry.register(Box::new(audit_count.clone())).unwrap();
        registry.register(Box::new(http_count.clone())).unwrap();
        registry.register(Box::new(http_latency.clone())).unwrap();

        Mutex::new(Metrics {
            registry,
            process_cpu_usage,
            process_resident_memory,

            audit_from: Utc::now(),
            audit_count,
            http_count,
            http_latency,
        })
    };
}

impl Metrics {
    pub fn http_metrics() -> (IntCounterVec, HistogramVec) {
        let metrics = METRICS.lock().unwrap();
        (metrics.http_count.clone(), metrics.http_latency.clone())
    }

    pub fn read(driver: &dyn Driver, service: Option<&Service>) -> DriverResult<String> {
        let mut metrics = METRICS.lock().unwrap();
        let audit_metrics =
            driver.audit_read_metrics(&metrics.audit_from, service.map(|s| &s.id))?;

        // TODO(refactor): Group by status codes for audit types.
        metrics.audit_from = Utc::now();
        for (path, count) in audit_metrics.iter() {
            metrics
                .audit_count
                .with_label_values(&[path])
                .inc_by(*count);
        }

        Self::sysinfo(&metrics.process_cpu_usage, &metrics.process_resident_memory)?;
        Self::registry_encode(&metrics.registry)
    }

    pub fn sysinfo(
        process_cpu_usage: &Counter,
        process_resident_memory: &IntCounter,
    ) -> DriverResult<()> {
        // TODO(feature): Support more process/other metrics, check units.
        // <https://prometheus.io/docs/instrumenting/writing_clientlibs/#standard-and-runtime-collectors>
        let mut system = SYSTEM.lock().unwrap();
        let pid = sysinfo::get_current_pid().unwrap();
        system.refresh_process(pid);
        let p = system.get_process(pid).unwrap();

        process_cpu_usage.inc_by(f64::from(p.cpu_usage()));
        let memory_bytes: i64 = (p.memory() * 1024)
            .try_into()
            .map_err(|_e| DriverError::Metrics)?;
        process_resident_memory.inc_by(memory_bytes);

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
