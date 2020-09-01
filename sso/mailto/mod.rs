use crate::internal::*;
use lettre::{
    smtp::authentication::{Credentials, Mechanism},
    ClientSecurity, ClientTlsParameters, SmtpClient, Transport,
};
use lettre_email::Email;
use native_tls::{Protocol, TlsConnector};
use tokio::prelude::*;

/// Mailto
#[derive(Clone)]
pub struct Mailto {
    pub config: Arc<ConfigMailto>,
    pub file: Option<Arc<Mutex<tokio::fs::File>>>,
    pub smtp: Option<MailtoSmtp>,
    pub opentelemetry: Arc<ServerOpentelemetry>,
}

/// Mailto SMTP
#[derive(Clone)]
pub struct MailtoSmtp {
    pub from: String,
    pub client: SmtpClient,
}

/// Mailto Send
#[derive(Debug, Clone, Serialize)]
pub struct MailtoSend {
    to: String,
    subject: String,
    text: String,
}

impl Mailto {
    /// Create mailto from configuration
    pub async fn from_config(
        config: ConfigMailto,
        opentelemetry: Arc<ServerOpentelemetry>,
    ) -> Self {
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

        Self {
            config: Arc::new(config),
            file: file.map(|x| Arc::new(Mutex::new(x))),
            smtp,
            opentelemetry,
        }
    }

    /// Send mail
    pub async fn send(&self, send: MailtoSend) -> Result<()> {
        match self.send_inner(send).await {
            Ok(_) => {
                self.opentelemetry.mailto_ok_count.add(1);
                Ok(())
            }
            Err(e) => {
                self.opentelemetry.mailto_err_count.add(1);
                Err(e)
            }
        }
    }

    async fn send_inner(&self, send: MailtoSend) -> Result<()> {
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

    async fn smtp_send(&self, smtp: MailtoSmtp, send: MailtoSend) -> Result<()> {
        blocking(move || {
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

    fn smtp_client(config: &ConfigMailtoSmtp) -> Result<SmtpClient> {
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

impl MailtoSend {
    /// Create new mail to send
    pub fn new(to: &str, subject: &str, text: &str) -> Self {
        Self {
            to: to.to_string(),
            subject: subject.to_string(),
            text: text.to_string(),
        }
    }
}
