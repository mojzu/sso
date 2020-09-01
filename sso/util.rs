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

pub mod validate {
    use crate::internal::*;

    pub fn email_vec(values: &Vec<String>) -> std::result::Result<(), ValidationError> {
        for value in values {
            if !validator::validate_email(value) {
                return Err(ValidationError::new("email_invalid"));
            }
        }
        Ok(())
    }

    pub fn id(value: i64) -> std::result::Result<(), ValidationError> {
        if value < 1 {
            return Err(ValidationError::new("id_invalid"));
        }
        Ok(())
    }

    pub fn id_vec(values: &Vec<i64>) -> std::result::Result<(), ValidationError> {
        for value in values {
            id(*value)?;
        }
        Ok(())
    }

    pub fn audit_type(value: &str) -> std::result::Result<(), ValidationError> {
        if value.len() < 1 || value.len() > 100 {
            return Err(ValidationError::new("audit_type_invalid"));
        }
        Ok(())
    }

    pub fn audit_type_vec(values: &Vec<String>) -> std::result::Result<(), ValidationError> {
        for value in values {
            audit_type(value)?;
        }
        Ok(())
    }

    pub fn audit_subject(value: &str) -> std::result::Result<(), ValidationError> {
        if value.len() < 1 || value.len() > 1000 {
            return Err(ValidationError::new("audit_subject_invalid"));
        }
        Ok(())
    }

    pub fn audit_subject_vec(values: &Vec<String>) -> std::result::Result<(), ValidationError> {
        for value in values {
            audit_subject(value)?;
        }
        Ok(())
    }

    pub fn client_id(value: &str) -> std::result::Result<(), ValidationError> {
        if value.len() < 1 || value.len() > 100 {
            return Err(ValidationError::new("client_id_invalid"));
        }
        Ok(())
    }

    pub fn csrf_token(value: &str) -> std::result::Result<(), ValidationError> {
        if value.len() < 1 || value.len() > 500 {
            return Err(ValidationError::new("csrf_token_invalid"));
        }
        Ok(())
    }

    pub fn code(value: &str) -> std::result::Result<(), ValidationError> {
        if value.len() < 1 || value.len() > 1000 {
            return Err(ValidationError::new("code_invalid"));
        }
        Ok(())
    }

    pub fn state(value: &str) -> std::result::Result<(), ValidationError> {
        if value.len() < 1 || value.len() > 1000 {
            return Err(ValidationError::new("state_invalid"));
        }
        Ok(())
    }

    pub fn token(value: &str) -> std::result::Result<(), ValidationError> {
        if value.len() < 1 || value.len() > 1000 {
            return Err(ValidationError::new("token_invalid"));
        }
        Ok(())
    }

    pub fn scope(value: &str) -> std::result::Result<(), ValidationError> {
        if value.len() > 1000 {
            return Err(ValidationError::new("scope_invalid"));
        }
        Ok(())
    }

    pub fn oauth2_provider(value: &str) -> std::result::Result<(), ValidationError> {
        if value.len() < 1 || value.len() > 20 {
            return Err(ValidationError::new("oauth2_provider_invalid"));
        }
        Ok(())
    }

    pub fn locale(value: &str) -> std::result::Result<(), ValidationError> {
        if value.len() > 100 {
            return Err(ValidationError::new("locale_invalid"));
        }
        Ok(())
    }

    pub fn timezone(value: &str) -> std::result::Result<(), ValidationError> {
        if value.len() > 500 {
            return Err(ValidationError::new("timezone_invalid"));
        }
        Ok(())
    }

    pub(crate) const PASSWORD_MIN: usize = 8;
    pub(crate) const PASSWORD_MAX: usize = 100;

    pub fn password(value: &str) -> std::result::Result<(), ValidationError> {
        if value.len() < PASSWORD_MIN || value.len() > PASSWORD_MAX {
            return Err(ValidationError::new("password_invalid"));
        }
        Ok(())
    }

    pub(crate) const NAME_MIN: usize = 1;
    pub(crate) const NAME_MAX: usize = 100;

    pub fn name(value: &str) -> std::result::Result<(), ValidationError> {
        if value.len() < NAME_MIN || value.len() > NAME_MAX {
            return Err(ValidationError::new("name_invalid"));
        }
        Ok(())
    }
}
