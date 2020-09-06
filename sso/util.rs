//! # Utilities
use crate::internal::*;

/// Crate Name
pub static NAME: &str = env!("CARGO_PKG_NAME");

/// Crate Version
pub static VERSION: &str = env!("CARGO_PKG_VERSION");

/// HTTP Client User Agent
pub static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// Initialise panic for single-line JSON output
pub fn init_panic(pretty: bool) {
    if !pretty {
        std::panic::set_hook(Box::new(|info| {
            let location = info.location().unwrap();
            let message = match info.payload().downcast_ref::<&'static str>() {
                Some(s) => *s,
                None => match info.payload().downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "Box<Any>",
                },
            };

            let out = serde_json::to_string(&json!({
                "time": chrono::Utc::now().to_rfc3339(),
                "level": log::Level::Error.to_string(),
                "file": format!("{}:{}:{}", location.file(), location.line(), location.column()),
                "message": message,
                "name": NAME,
                "version": VERSION,
            }))
            .expect("log format failure");
            eprintln!("{}", out);
        }));
    }
}

/// Initialise logging using `env_logger` for single-line JSON output
pub fn init_log(pretty: bool) {
    use std::io::Write;

    let mut builder = env_logger::Builder::from_default_env();
    let start_time = Utc::now();

    builder.format(move |buf, record| {
        let time = chrono::Utc::now();
        let level = record.level().to_string();
        let file = record.file();
        let line = record.line();
        let message = record.args();

        let out = if pretty {
            let run_time = time - start_time;
            Ok(format!(
                "{} {}:{} {}s\r\n{}\r\n",
                level,
                file.unwrap_or("none"),
                line.unwrap_or(0),
                run_time.num_seconds(),
                message
            ))
        } else {
            serde_json::to_string(&json!({
                "time": time.to_rfc3339(),
                "level": level,
                "target": record.target(),
                "module_path": record.module_path(),
                "file": format!("{}:{}", file.unwrap_or("none"), line.unwrap_or(0)),
                "message": message,
                "name": NAME,
                "version": VERSION,
            }))
        }
        .expect("log format failure");

        if pretty {
            let style = buf.default_level_style(record.level());
            writeln!(buf, "{}", style.value(out))
        } else {
            writeln!(buf, "{}", out)
        }
    });

    builder.init();
}

/// Run a blocking closure on threadpool.
pub async fn blocking<T, F>(f: F) -> Result<T>
where
    F: Send + FnOnce() -> Result<T> + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(move || f())
        .await
        .unwrap_or_else(|e| panic!("error running async task: {:?}", e))
}
