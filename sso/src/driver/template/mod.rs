use crate::{AuditMeta, DriverError, DriverResult, Service, User};
use chrono::{DateTime, Utc};
use handlebars::Handlebars;

const EMAIL_REGISTER: &str = "email_register";
const EMAIL_REGISTER_CONFIRM: &str = "email_register_confirm";
const EMAIL_RESET_PASSWORD: &str = "email_reset_password";
const EMAIL_RESET_PASSWORD_CONFIRM: &str = "email_reset_password_confirm";
const EMAIL_UPDATE_EMAIL: &str = "email_update_email";
const EMAIL_UPDATE_PASSWORD: &str = "email_update_password";

lazy_static! {
    static ref HANDLEBARS: Handlebars = {
        let mut handlebars = Handlebars::new();

        handlebars
            .register_template_string(EMAIL_REGISTER, include_str!("email_register.hbs"))
            .unwrap();
        handlebars
            .register_template_string(
                EMAIL_REGISTER_CONFIRM,
                include_str!("email_register_confirm.hbs"),
            )
            .unwrap();
        handlebars
            .register_template_string(
                EMAIL_RESET_PASSWORD,
                include_str!("email_reset_password.hbs"),
            )
            .unwrap();
        handlebars
            .register_template_string(
                EMAIL_RESET_PASSWORD_CONFIRM,
                include_str!("email_reset_password_confirm.hbs"),
            )
            .unwrap();
        handlebars
            .register_template_string(EMAIL_UPDATE_EMAIL, include_str!("email_update_email.hbs"))
            .unwrap();
        handlebars
            .register_template_string(
                EMAIL_UPDATE_PASSWORD,
                include_str!("email_update_password.hbs"),
            )
            .unwrap();

        handlebars
    };
}

/// Template email audit data.
#[derive(Debug, Serialize)]
struct TemplateEmailAudit {
    datetime: DateTime<Utc>,
    user_agent: String,
    remote: String,
    forwarded: Option<String>,
}

impl TemplateEmailAudit {
    pub fn new(audit: &AuditMeta) -> Self {
        Self {
            datetime: Utc::now(),
            user_agent: audit.user_agent().to_owned(),
            remote: audit.remote().to_owned(),
            forwarded: audit.forwarded().map(|x| x.to_owned()),
        }
    }
}

/// Template email service data.
#[derive(Debug, Serialize)]
struct TemplateEmailService {
    name: String,
    url: String,
    text: String,
}

impl TemplateEmailService {
    pub fn new(service: &Service) -> Self {
        Self {
            name: service.name.to_owned(),
            url: service.url.to_owned(),
            text: service.user_email_text.to_owned(),
        }
    }
}

/// Template email generic parameters.
#[derive(Debug, Serialize)]
struct TemplateEmailGeneric {
    user_email: String,
    url: String,
    audit: TemplateEmailAudit,
    service: TemplateEmailService,
}

impl TemplateEmailGeneric {
    pub fn new<UE, U>(user_email: UE, url: U, audit: &AuditMeta, service: &Service) -> Self
    where
        UE: Into<String>,
        U: Into<String>,
    {
        Self {
            user_email: user_email.into(),
            url: url.into(),
            audit: TemplateEmailAudit::new(audit),
            service: TemplateEmailService::new(service),
        }
    }
}

/// Template email update email parameters.
#[derive(Debug, Serialize)]
struct TemplateEmailUpdateEmail {
    user_old_email: String,
    user_email: String,
    url: String,
    audit: TemplateEmailAudit,
    service: TemplateEmailService,
}

impl TemplateEmailUpdateEmail {
    pub fn new<OE, UE, U>(
        user_old_email: OE,
        user_email: UE,
        url: U,
        audit: &AuditMeta,
        service: &Service,
    ) -> Self
    where
        OE: Into<String>,
        UE: Into<String>,
        U: Into<String>,
    {
        Self {
            user_old_email: user_old_email.into(),
            user_email: user_email.into(),
            url: url.into(),
            audit: TemplateEmailAudit::new(audit),
            service: TemplateEmailService::new(service),
        }
    }
}

/// Template email.
#[derive(Debug)]
pub struct TemplateEmail {
    pub to_email: String,
    pub to_name: String,
    pub from_name: String,
    pub subject: String,
    pub text: String,
}

impl TemplateEmail {
    fn new<T, S>(to_email: T, to_name: T, from_name: T, subject: S, text: String) -> Self
    where
        T: Into<String>,
        S: Into<String>,
    {
        Self {
            to_email: to_email.into(),
            to_name: to_name.into(),
            from_name: from_name.into(),
            subject: subject.into(),
            text,
        }
    }

    /// Render register email template.
    pub fn email_register(
        service: &Service,
        user: &User,
        token: &str,
        audit: &AuditMeta,
    ) -> DriverResult<Self> {
        let url = service.provider_local_callback_url(
            "register",
            json!({
                "email": user.email,
                "token": token,
            }),
        )?;

        let text = HANDLEBARS
            .render(
                EMAIL_REGISTER,
                &TemplateEmailGeneric::new(&user.email, url.as_str(), audit, service),
            )
            .map_err(DriverError::HandlebarsRender)?;
        Ok(Self::new(
            &user.email,
            &user.name,
            &service.name,
            "Registration Request",
            text,
        ))
    }

    /// Render register confirm email template.
    pub fn email_register_confirm(
        service: &Service,
        user: &User,
        token: &str,
        audit: &AuditMeta,
    ) -> DriverResult<Self> {
        let url = service.provider_local_callback_url(
            "register_confirm",
            json!({
                "email": user.email,
                "token": token,
            }),
        )?;

        let text = HANDLEBARS
            .render(
                EMAIL_REGISTER_CONFIRM,
                &TemplateEmailGeneric::new(&user.email, url.as_str(), audit, service),
            )
            .map_err(DriverError::HandlebarsRender)?;
        Ok(Self::new(
            &user.email,
            &user.name,
            &service.name,
            "Registration Confirmed",
            text,
        ))
    }

    /// Render reset password email template.
    pub fn email_reset_password(
        service: &Service,
        user: &User,
        token: &str,
        audit: &AuditMeta,
    ) -> DriverResult<Self> {
        let url = service.provider_local_callback_url(
            "reset_password",
            json!({
                "email": user.email,
                "token": token,
            }),
        )?;

        let text = HANDLEBARS
            .render(
                EMAIL_RESET_PASSWORD,
                &TemplateEmailGeneric::new(&user.email, url.as_str(), audit, service),
            )
            .map_err(DriverError::HandlebarsRender)?;
        Ok(Self::new(
            &user.email,
            &user.name,
            &service.name,
            "Password Reset Request",
            text,
        ))
    }

    /// Render reset password confirm email template.
    pub fn email_reset_password_confirm(
        service: &Service,
        user: &User,
        token: &str,
        audit: &AuditMeta,
    ) -> DriverResult<Self> {
        let url = service.provider_local_callback_url(
            "reset_password_confirm",
            json!({
                "email": user.email,
                "token": token,
            }),
        )?;

        let text = HANDLEBARS
            .render(
                EMAIL_RESET_PASSWORD_CONFIRM,
                &TemplateEmailGeneric::new(&user.email, url.as_str(), audit, service),
            )
            .map_err(DriverError::HandlebarsRender)?;
        Ok(Self::new(
            &user.email,
            &user.name,
            &service.name,
            "Password Reset Confirmed",
            text,
        ))
    }

    /// Render update email email template.
    pub fn email_update_email(
        service: &Service,
        user: &User,
        old_email: &str,
        token: &str,
        audit: &AuditMeta,
    ) -> DriverResult<Self> {
        let url = service.provider_local_callback_url(
            "update_email",
            json!({
                "email": user.email,
                "old_email": old_email,
                "token": token,
            }),
        )?;

        let text = HANDLEBARS
            .render(
                EMAIL_UPDATE_EMAIL,
                &TemplateEmailUpdateEmail::new(
                    old_email,
                    &user.email,
                    url.as_str(),
                    audit,
                    service,
                ),
            )
            .map_err(DriverError::HandlebarsRender)?;
        Ok(Self::new(
            &user.email,
            &user.name,
            &service.name,
            "Email Address Updated",
            text,
        ))
    }

    /// Render password email template.
    pub fn email_update_password(
        service: &Service,
        user: &User,
        token: &str,
        audit: &AuditMeta,
    ) -> DriverResult<Self> {
        let url = service.provider_local_callback_url(
            "update_password",
            json!({
                "email": user.email,
                "token": token,
            }),
        )?;

        let text = HANDLEBARS
            .render(
                EMAIL_UPDATE_PASSWORD,
                &TemplateEmailGeneric::new(&user.email, url.as_str(), audit, service),
            )
            .map_err(DriverError::HandlebarsRender)?;
        Ok(Self::new(
            &user.email,
            &user.name,
            &service.name,
            "Password Updated",
            text,
        ))
    }
}
