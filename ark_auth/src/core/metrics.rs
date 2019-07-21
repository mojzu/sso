use crate::core::audit::AuditBuilder;
use crate::core::{Error, Service};
use crate::driver;
use prometheus::{Encoder, IntCounter, Opts, Registry, TextEncoder};

pub fn read(
    driver: &driver::Driver,
    service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    registry: &Registry,
) -> Result<String, Error> {
    // TODO(refactor): More efficient way of handling audit metrics.
    let audit_registry = Registry::new();
    let audit_metrics = driver
        .audit_read_metrics(service_mask.map(|s| s.id.as_ref()))
        .map_err(Error::Driver)?;
    for (path, count) in audit_metrics.iter() {
        // TODO(refactor): Get help strings from somewhere.
        let opts = Opts::new(path.to_owned(), "...".to_owned());
        let counter = IntCounter::with_opts(opts).unwrap();
        counter.inc_by(*count);
        audit_registry.register(Box::new(counter)).unwrap();
    }

    let encoded = encode_registry(registry)?;
    let audit_encoded = encode_registry(&audit_registry)?;
    let text = format!("{}\n{}", encoded, audit_encoded);
    Ok(text)
}

fn encode_registry(registry: &Registry) -> Result<String, Error> {
    registry.gather();

    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metrics = registry.gather();
    encoder.encode(&metrics, &mut buffer).unwrap();
    Ok(String::from_utf8(buffer).unwrap())
}
