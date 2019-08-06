use crate::core::audit::AuditBuilder;
use crate::core::{Error, Service};
use crate::driver;
use prometheus::{Counter, Encoder, IntCounter, IntCounterVec, Opts, Registry, TextEncoder};
use std::sync::Mutex;
use sysinfo::{ProcessExt, System, SystemExt};

lazy_static! {
    static ref SYSTEM: Mutex<System> = { Mutex::new(System::new()) };
}

pub fn name(name: &str) -> String {
    let prefix = crate_name!();
    format!("{}_{}", prefix, name)
}

pub fn sysinfo_encoded() -> Result<String, Error> {
    let registry = Registry::new();

    // TODO(feature): Support more process/other metrics.
    // <https://prometheus.io/docs/instrumenting/writing_clientlibs/#standard-and-runtime-collectors>
    let mut system = SYSTEM.lock().unwrap();
    let pid = sysinfo::get_current_pid().unwrap();
    system.refresh_process(pid);
    let p = system.get_process(pid).unwrap();

    let cpu_usage_counter = Counter::new("process_cpu_usage", "CPU usage (%).").unwrap();
    registry
        .register(Box::new(cpu_usage_counter.clone()))
        .unwrap();
    cpu_usage_counter.inc_by(p.cpu_usage() as f64);

    let memory_counter = IntCounter::new(
        "process_resident_memory_bytes",
        "Resident memory size in bytes.",
    )
    .unwrap();
    registry.register(Box::new(memory_counter.clone())).unwrap();
    let memory_bytes = p.memory() * 1024;
    memory_counter.inc_by(memory_bytes as i64);

    encode_registry(&registry)
}

pub fn read(
    driver: &driver::Driver,
    service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    registry: &Registry,
) -> Result<String, Error> {
    let audit_metrics = driver
        .audit_read_metrics(service_mask.map(|s| s.id.as_ref()))
        .map_err(Error::Driver)?;

    // TODO(refactor): More efficient way of handling audit metrics read.
    // Keep audit registry and counter alive, only query metrics since last query.
    let audit_registry = Registry::new();
    let opts = Opts::new(name("audit"), "Audit log counter".to_owned());
    let counter = IntCounterVec::new(opts, &["path"]).unwrap();
    audit_registry.register(Box::new(counter.clone())).unwrap();
    for (path, count) in audit_metrics.iter() {
        counter.with_label_values(&[path]).inc_by(*count);
    }

    let sysinfo_encoded = sysinfo_encoded()?;
    let encoded = encode_registry(registry)?;
    let audit_encoded = encode_registry(&audit_registry)?;
    let text = format!("{}\n{}\n{}", sysinfo_encoded, encoded, audit_encoded);
    Ok(text)
}

fn encode_registry(registry: &Registry) -> Result<String, Error> {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metrics = registry.gather();
    encoder.encode(&metrics, &mut buffer).unwrap();
    Ok(String::from_utf8(buffer).unwrap())
}
