use crate::core::audit::AuditBuilder;
use crate::core::{Error, Service};
use crate::driver;
use prometheus::{Encoder, IntCounterVec, Opts, Registry, TextEncoder};

pub fn name(name: &str) -> String {
    let prefix = crate_name!();
    format!("{}_{}", prefix, name)
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

    let audit_registry = Registry::new();
    let opts = Opts::new(name("audit"), "Audit log counter".to_owned());
    let counter = IntCounterVec::new(opts, &["path"]).unwrap();
    audit_registry.register(Box::new(counter.clone())).unwrap();
    for (path, count) in audit_metrics.iter() {
        counter.with_label_values(&[path]).inc_by(*count);
    }

    let encoded = encode_registry(registry)?;
    let audit_encoded = encode_registry(&audit_registry)?;
    let text = format!("{}\n{}", encoded, audit_encoded);
    Ok(text)
}

fn encode_registry(registry: &Registry) -> Result<String, Error> {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metrics = registry.gather();
    encoder.encode(&metrics, &mut buffer).unwrap();
    Ok(String::from_utf8(buffer).unwrap())
}
