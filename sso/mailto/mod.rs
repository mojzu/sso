//! # Mailto
use crate::internal::*;
use lettre::{
    smtp::authentication::{Credentials, Mechanism},
    ClientSecurity, ClientTlsParameters, SmtpClient, Transport,
};
use lettre_email::Email;
use native_tls::{Protocol, TlsConnector};
use tokio::prelude::*;

/// Mailto Configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub stdout: ConfigStdout,
    #[serde(default)]
    pub file: ConfigFile,
    pub smtp: Option<ConfigSmtp>,
}

/// Mailto
#[derive(Clone)]
pub struct Mailto {
    config: Arc<Config>,
    file: Option<Arc<Mutex<tokio::fs::File>>>,
    smtp: Option<MailtoSmtp>,
    opentelemetry: Arc<MailtoOpentelemetry>,
}

/// Mailto Send
#[derive(Debug, Clone, Serialize)]
pub struct Send {
    to: String,
    subject: String,
    text: String,
}

/// Create mailto from configuration
pub async fn from_config(metrics: &metrics::Metrics, config: Config) -> Result<Mailto> {
    Mailto::from_config(metrics, config).await
}

impl Mailto {
    /// Returns configuration
    pub fn config(&self) -> &Config {
        self.config.as_ref()
    }

    /// Create mailto from configuration
    pub async fn from_config(metrics: &metrics::Metrics, config: Config) -> Result<Self> {
        let file = if let Some(file) = &config.file.file {
            let f = tokio::fs::File::create(file).await.unwrap();
            Some(f)
        } else {
            None
        };

        let smtp = if let Some(smtp) = &config.smtp {
            let client = Self::smtp_client(smtp).unwrap();
            Some(MailtoSmtp {
                from: smtp.from.to_string(),
                client,
            })
        } else {
            None
        };

        let opentelemetry = MailtoOpentelemetry {
            ok_count: metrics
                .meter()
                .u64_counter("mailto_ok_count")
                .with_description("Total number of mailto send successes.")
                .init()
                .bind(&[]),
            err_count: metrics
                .meter()
                .u64_counter("mailto_err_count")
                .with_description("Total number of mailto send errors.")
                .init()
                .bind(&[]),
        };

        Ok(Self {
            config: Arc::new(config),
            file: file.map(|x| Arc::new(Mutex::new(x))),
            smtp,
            opentelemetry: Arc::new(opentelemetry),
        })
    }

    /// Build mail
    pub fn build(&self, to: &str, subject: &str, text: &str) -> Send {
        Send {
            to: to.to_string(),
            subject: subject.to_string(),
            text: text.to_string(),
        }
    }

    /// Send mail
    pub async fn send(&self, send: Send) -> Result<()> {
        match self.send_inner(send).await {
            Ok(_) => {
                self.opentelemetry.ok_count.add(1);
                Ok(())
            }
            Err(e) => {
                self.opentelemetry.err_count.add(1);
                Err(e)
            }
        }
    }

    async fn send_inner(&self, send: Send) -> Result<()> {
        let json_out = serde_json::to_string(&send).unwrap();

        if self.config.stdout.enable {
            println!("{}", json_out);
        }
        if let Some(file) = self.file.as_ref() {
            let mut file = file.lock().unwrap();
            let line = format!("{}\n", json_out);
            file.write(line.as_bytes()).await.unwrap();
        }
        if let Some(smtp) = self.smtp.as_ref() {
            self.smtp_send(smtp.clone(), send.clone()).await?;
        }
        Ok(())
    }

    async fn smtp_send(&self, smtp: MailtoSmtp, send: Send) -> Result<()> {
        util::blocking(move || {
            let email = Email::builder()
                .from(smtp.from)
                .to(send.to)
                .subject(send.subject)
                .text(send.text)
                .build()?;

            let mut transport = smtp.client.transport();
            transport.send(email.into())?;
            Ok(())
        })
        .await
    }

    fn smtp_client(config: &ConfigSmtp) -> Result<SmtpClient> {
        let mut tls_builder = TlsConnector::builder();
        tls_builder.min_protocol_version(Some(Protocol::Tlsv10));
        let tls_parameters =
            ClientTlsParameters::new(config.host.to_string(), tls_builder.build().unwrap());
        let mut client = SmtpClient::new(
            (config.host.as_ref(), config.port),
            ClientSecurity::Required(tls_parameters),
        )?;

        client = if let Some(login) = &config.login {
            client
                .authentication_mechanism(Mechanism::Login)
                .credentials(Credentials::new(
                    login.user.to_string(),
                    login.password.to_string(),
                ))
        } else {
            client
        };

        Ok(client)
    }
}

/// Stdout Mailto Configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigStdout {
    pub enable: bool,
}

/// File Mailto Configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub file: Option<String>,
}

/// SMTP Mailto Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSmtp {
    pub host: String,
    pub port: u16,
    pub from: String,
    pub login: Option<ConfigSmtpLogin>,
}

/// SMTP Login Mailto Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSmtpLogin {
    pub user: String,
    pub password: String,
}

/// Mailto SMTP
#[derive(Clone)]
struct MailtoSmtp {
    from: String,
    client: SmtpClient,
}

/// Mailto Opentelemetry
#[derive(Debug)]
struct MailtoOpentelemetry {
    ok_count: BoundCounter<'static, u64>,
    err_count: BoundCounter<'static, u64>,
}
