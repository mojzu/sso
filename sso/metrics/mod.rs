//! # Metrics
use crate::internal::*;

/// Metrics Configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Name
    #[serde(default = "default_as_sso")]
    pub name: String,
}

/// Metrics
#[derive(Clone)]
pub struct Metrics {
    config: Arc<Config>,
    exporter: Arc<opentelemetry_prometheus::PrometheusExporter>,
    meter: Arc<opentelemetry::metrics::Meter>,
}

/// Create metrics from configuration
pub fn from_config(config: Config) -> Result<Metrics> {
    Metrics::from_config(config)
}

impl Metrics {
    /// Returns configuration
    pub fn config(&self) -> &Config {
        self.config.as_ref()
    }

    /// Create metrics from configuration
    pub fn from_config(config: Config) -> Result<Self> {
        let exporter = opentelemetry_prometheus::exporter().init();
        let meter = opentelemetry::global::meter(&config.name);

        Ok(Self {
            config: Arc::new(config),
            exporter: Arc::new(exporter),
            meter: Arc::new(meter),
        })
    }

    /// Returns opentelemetry meter
    pub fn meter(&self) -> &opentelemetry::metrics::Meter {
        self.meter.as_ref()
    }

    /// Returns exposition format type and string
    pub fn encode(&self) -> (String, Vec<u8>) {
        use prometheus::{Encoder, TextEncoder};

        let mut buffer = vec![];
        let encoder = TextEncoder::new();

        let mut metric_families = prometheus::gather();
        let mut ot_metric_families = self.exporter.registry().gather();
        metric_families.append(&mut ot_metric_families);
        encoder.encode(&metric_families, &mut buffer).unwrap();

        (encoder.format_type().to_string(), buffer)
    }
}

impl std::fmt::Debug for Metrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Metrics {{ config, exporter, meter }}")
    }
}
