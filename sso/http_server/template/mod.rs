use crate::internal::*;

#[derive(Debug, Serialize)]
pub(crate) struct ContextError {
    code: String,
    description: String,
    audit_id: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct ContextLen {
    minlength: usize,
    maxlength: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct ContextOauth2Providers {
    sso: bool,
    microsoft: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct Context {
    client_id: String,
    client_name: String,
    client_uri: Url,
    client_domain: String,
    client_register_enable: bool,
    csrf_token: Option<String>,
    error: Option<ContextError>,
    message: Option<String>,
    password: ContextLen,
    email: ContextLen,
    name: ContextLen,
    oauth2_providers: ContextOauth2Providers,
}

impl Context {
    pub fn build(oauth2_providers: &ConfigOauth2Providers, client: &Client) -> Self {
        Self {
            client_id: client.client_id(),
            client_name: client.client_name(),
            client_uri: client.client_uri(),
            client_domain: client.client_uri().domain().unwrap().to_string(),
            client_register_enable: client.register_enable,
            csrf_token: None,
            error: None,
            message: None,
            password: ContextLen {
                minlength: validate::PASSWORD_MIN,
                maxlength: validate::PASSWORD_MAX,
            },
            email: ContextLen {
                minlength: 1,
                maxlength: 1000,
            },
            name: ContextLen {
                minlength: validate::NAME_MIN,
                maxlength: validate::NAME_MAX,
            },
            oauth2_providers: ContextOauth2Providers {
                sso: oauth2_providers.sso.is_some(),
                microsoft: oauth2_providers.microsoft.is_some(),
            },
        }
    }

    pub fn csrf_token(&mut self, csrf_token: String) {
        self.csrf_token = Some(csrf_token);
    }

    pub fn error<T: Into<ContextError>>(&mut self, err: T) {
        self.error = Some(err.into());
    }

    pub fn message(&mut self, message: String) {
        self.message = Some(message);
    }
}

impl From<(i64, oauth2::ErrorResponse)> for ContextError {
    fn from(e: (i64, oauth2::ErrorResponse)) -> Self {
        Self {
            code: e.1.error().as_str().to_string(),
            description: e.1.error_description().to_string(),
            audit_id: e.0,
        }
    }
}

pub const TEMPLATE_HTML_WRAPPER: &str = r#"
{{{ content }}}
"#;

pub const TEMPLATE_MAIL_TEXT_WRAPPER: &str = r#"
{{{ content }}}
"#;

pub const TEMPLATE_ERROR: &str = include_str!("error.hbs");

pub const TEMPLATE_AUTH: &str = include_str!("auth.hbs");

pub const TEMPLATE_AUTH_PASSWORD_RESET: &str = include_str!("password_reset/request.hbs");

pub const TEMPLATE_AUTH_PASSWORD_RESET_ACCEPT: &str = include_str!("password_reset/accept.hbs");

pub const TEMPLATE_AUTH_PASSWORD_RESET_ACCEPT_OK: &str =
    include_str!("password_reset/accept_ok.hbs");

pub const TEMPLATE_AUTH_PASSWORD_RESET_REJECT: &str = include_str!("password_reset/reject.hbs");

pub const TEMPLATE_AUTH_PASSWORD_RESET_REJECT_OK: &str =
    include_str!("password_reset/reject_ok.hbs");

pub const TEMPLATE_AUTH_EMAIL_UPDATE: &str = include_str!("email_update/request.hbs");

pub const TEMPLATE_AUTH_EMAIL_UPDATE_OK: &str = include_str!("email_update/request_ok.hbs");

pub const TEMPLATE_AUTH_PASSWORD_UPDATE: &str = include_str!("password_update/request.hbs");

pub const TEMPLATE_AUTH_PASSWORD_UPDATE_OK: &str = include_str!("password_update/request_ok.hbs");

pub const TEMPLATE_AUTH_REGISTER: &str = include_str!("register/request.hbs");

pub const TEMPLATE_AUTH_REGISTER_ACCEPT: &str = include_str!("register/accept.hbs");

pub const TEMPLATE_AUTH_REGISTER_ACCEPT_OK: &str = include_str!("register/accept_ok.hbs");

pub const TEMPLATE_AUTH_REGISTER_REJECT: &str = include_str!("register/reject.hbs");

pub const TEMPLATE_AUTH_REGISTER_REJECT_OK: &str = include_str!("register/reject_ok.hbs");

pub const TEMPLATE_AUTH_LOGOUT: &str = include_str!("logout.hbs");

pub const TEMPLATE_AUTH_DELETE: &str = include_str!("delete/request.hbs");

pub const TEMPLATE_AUTH_DELETE_OK: &str = include_str!("delete/request_ok.hbs");

pub const TEMPLATE_AUTH_DELETE_ACCEPT: &str = include_str!("delete/accept.hbs");

pub const TEMPLATE_AUTH_DELETE_ACCEPT_OK: &str = include_str!("delete/accept_ok.hbs");

pub const TEMPLATE_AUTH_DELETE_REJECT: &str = include_str!("delete/reject.hbs");

pub const TEMPLATE_AUTH_DELETE_REJECT_OK: &str = include_str!("delete/reject_ok.hbs");

pub const TEMPLATE_MAIL_PASSWORD_RESET: &str = r#"
Password Reset Request

You are receiving this email because a password reset request was made for the following user.

{{user_email}}

If you made this request, click the following link.

{{{uri_accept}}}

If you did not make this request, click the following link.

{{{uri_reject}}}

This request was made by the following client.

{{client_name}}
{{{client_uri}}}

{{#if audit}}More technical information about this request.

Timestamp: {{audit.timestamp}}
Peer Address: {{audit.peer}}
Remote Address: {{audit.remote}}
User Agent: {{audit.user_agent}}{{/if}}
"#;

pub const TEMPLATE_MAIL_REGISTER: &str = r#"
Register Request

You are receiving this email because a registration request was made for the following email address.

{{user_email}}

If you made this request, click the following link.

{{{uri_accept}}}

If you did not make this request, click the following link.

{{{uri_reject}}}

This request was made by the following client.

{{client_name}}
{{{client_uri}}}

{{#if audit}}More technical information about this request.

Timestamp: {{audit.timestamp}}
Peer Address: {{audit.peer}}
Remote Address: {{audit.remote}}
User Agent: {{audit.user_agent}}{{/if}}
"#;

pub const TEMPLATE_MAIL_DELETE: &str = r#"
Delete Request

You are receiving this email because a request was made to delete the following user.

{{user_email}}

If you made this request, click the following link.

{{{uri_accept}}}

If you did not make this request, click the following link.

{{{uri_reject}}}

This request was made by the following client.

{{client_name}}
{{{client_uri}}}

{{#if audit}}More technical information about this request.

Timestamp: {{audit.timestamp}}
Peer Address: {{audit.peer}}
Remote Address: {{audit.remote}}
User Agent: {{audit.user_agent}}{{/if}}
"#;
