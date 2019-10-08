use crate::{AuditBuilder, CoreError, CoreResult, Driver, Service};
use chrono::{DateTime, Utc};
use prometheus::{Counter, Encoder, IntCounter, IntCounterVec, Opts, Registry, TextEncoder};
use std::{convert::TryInto, sync::Mutex};
use sysinfo::{ProcessExt, System, SystemExt};

struct Cache {
    pub from: DateTime<Utc>,
    pub audit_registry: Registry,
    pub audit_counter: IntCounterVec,
}

lazy_static! {
    static ref SYSTEM: Mutex<System> = { Mutex::new(System::new()) };
    static ref CACHE: Mutex<Cache> = {
        let audit_registry = Registry::new();
        let opts = Opts::new(Metrics::name("audit"), "Audit log counter".to_owned());
        let audit_counter = IntCounterVec::new(opts, &["path"]).unwrap();
        audit_registry
            .register(Box::new(audit_counter.clone()))
            .unwrap();

        Mutex::new(Cache {
            from: Utc::now(),
            audit_registry,
            audit_counter,
        })
    };
}

/// Metrics functions.
#[derive(Debug)]
pub struct Metrics;

impl Metrics {
    pub fn name(name: &str) -> String {
        let prefix = crate_name!();
        format!("{}_{}", prefix, name)
    }

    pub fn sysinfo_encoded() -> CoreResult<String> {
        let registry = Registry::new();

        // TODO(feature): Support more process/other metrics, check units.
        // <https://prometheus.io/docs/instrumenting/writing_clientlibs/#standard-and-runtime-collectors>
        let mut system = SYSTEM.lock().unwrap();
        let pid = sysinfo::get_current_pid().unwrap();
        system.refresh_process(pid);
        let p = system.get_process(pid).unwrap();

        let cpu_usage_counter = Counter::new("process_cpu_usage", "CPU usage (%).").unwrap();
        registry
            .register(Box::new(cpu_usage_counter.clone()))
            .unwrap();
        cpu_usage_counter.inc_by(f64::from(p.cpu_usage()));

        let memory_counter = IntCounter::new(
            "process_resident_memory_bytes",
            "Resident memory size in bytes.",
        )
        .unwrap();
        registry.register(Box::new(memory_counter.clone())).unwrap();
        let memory_bytes: i64 = (p.memory() * 1024)
            .try_into()
            .map_err(|_e| CoreError::Metrics)?;
        memory_counter.inc_by(memory_bytes);

        Metrics::encode_registry(&registry)
    }

    pub fn read(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        registry: &Registry,
    ) -> CoreResult<String> {
        let mut cache = CACHE.lock().unwrap();
        let audit_metrics = driver
            .audit_read_metrics(&cache.from, service_mask.map(|s| &s.id))
            .map_err(CoreError::Driver)?;
        cache.from = Utc::now();

        let audit_registry = &cache.audit_registry;
        let audit_counter = &cache.audit_counter;
        for (path, count) in audit_metrics.iter() {
            audit_counter.with_label_values(&[path]).inc_by(*count);
        }

        let sysinfo_encoded = Metrics::sysinfo_encoded()?;
        let encoded = Metrics::encode_registry(registry)?;
        let audit_encoded = Metrics::encode_registry(audit_registry)?;
        let text = format!("{}\n{}\n{}", sysinfo_encoded, encoded, audit_encoded);
        Ok(text)
    }

    fn encode_registry(registry: &Registry) -> CoreResult<String> {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metrics = registry.gather();
        encoder.encode(&metrics, &mut buffer).unwrap();
        Ok(String::from_utf8(buffer).unwrap())
    }
}
