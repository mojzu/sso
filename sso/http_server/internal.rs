pub use crate::{
    http_server::{client::*, error::*, template::*},
    internal::*,
};
pub use actix_http::ResponseError;
pub use paperclip::actix::{
    api_v2_operation,
    web::{self, Data, Form, HttpRequest, HttpResponse, Json, Path, Query},
};

use std::{fmt, time::SystemTime};

#[derive(Debug, Clone)]
pub struct UserLoginArgs {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct UserRegisterAcceptArgs {
    pub name: String,
    pub password: String,
    pub password_allow_reset: bool,
}

#[derive(Debug, Clone)]
pub struct UserRegisterPasswordArgs {
    pub password: String,
    pub password_confirm: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ServerRequest {
    pub time: SystemTime,
}

macro_rules! server_request {
    ($server:expr, $req:expr, $e:expr) => {
        $server.request($server.pre($req).await, $e.await).await;
    };
}

macro_rules! server_oauth2_validate {
    ($server:expr, $e:expr) => {
        match $e.validate() {
            Ok(_) => $e.into_inner(),
            Err(_e) => {
                $server.opentelemetry.validation_err_count.add(1);
                return Err(oauth2::ErrorResponse::invalid_request("validation failed"));
            }
        }
    };
}

macro_rules! server_validate {
    ($server:expr, $e:expr) => {
        match $e.validate() {
            Ok(_) => $e.into_inner(),
            Err(_e) => {
                $server.opentelemetry.validation_err_count.add(1);
                return Err(HttpError::BadRequest("validation failed".into()));
            }
        }
    };
}

macro_rules! server_oauth2_form_error {
    ($server:expr, $audit:expr, $client:expr, $template:expr, $e:expr) => {
        match $e.await {
            Ok(res) => {
                $audit.set_status_ok();
                $server.audit_insert($audit).await?;

                $server.opentelemetry.api_ok_count.add(1);
                Ok(res)
            }
            Err(e) => {
                $audit.set_status_err(&e);
                $audit.set_data_err(&e);
                let audit_id = $server.audit_insert($audit).await?;

                let context = $server
                    .template_csrf_error_context($client, (audit_id, e))
                    .await?;

                $server.opentelemetry.api_err_count.add(1);
                $server.response_template_context($client, $template, context)
            }
        }
    };
}

macro_rules! server_oauth2_error {
    ($server:expr, $audit:expr, $client:expr, $template:expr, $e:expr) => {
        match $e.await {
            Ok(res) => {
                $audit.set_status_ok();
                $server.audit_insert($audit).await?;

                $server.opentelemetry.api_ok_count.add(1);
                Ok(res)
            }
            Err(e) => {
                $audit.set_status_err(&e);
                $audit.set_data_err(&e);
                let audit_id = $server.audit_insert($audit).await?;

                let context = $server.template_error_context($client, (audit_id, e));

                $server.opentelemetry.api_err_count.add(1);
                $server.response_template_context($client, $template, context)
            }
        }
    };
}

impl HttpServer {
    pub(crate) async fn audit_insert(&self, audit: Audit) -> oauth2::Result<i64> {
        let audit = self
            .postgres
            .audit_insert(audit)
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;

        Ok(audit.id)
    }

    pub(crate) async fn request<T>(&self, pre: ServerRequest, res: T) -> T {
        self.opentelemetry
            .http_req_histogram
            .record(pre.time.elapsed().map_or(0.0, |d| d.as_secs_f64()));

        res
    }

    pub(crate) async fn pre(&self, _req: &HttpRequest) -> ServerRequest {
        self.opentelemetry.http_req_count.add(1);

        ServerRequest {
            time: SystemTime::now(),
        }
    }

    pub(crate) async fn csrf_token(&self, client: &Client) -> oauth2::Result<String> {
        self.postgres
            .csrf_insert(client)
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))
            .map(|csrf| csrf.token)
    }

    pub(crate) async fn csrf_verify(&self, client: &Client, token: String) -> oauth2::Result<()> {
        self.postgres
            .csrf_verify(client, RequestCsrf { token })
            .await
            .map_err(|e| oauth2::ErrorResponse::invalid_request(&e.to_string()))
    }

    /// Returns request identity
    pub(crate) async fn request_identity(
        &self,
        req: &actix_web::HttpRequest,
    ) -> actix_identity::Identity {
        use actix_web::{dev::Payload, FromRequest};
        actix_identity::Identity::from_request(&req, &mut Payload::None)
            .await
            .unwrap()
    }

    pub(crate) async fn request_identity_required(
        &self,
        audit: &mut Audit,
        req: &actix_web::HttpRequest,
    ) -> oauth2::Result<Uuid> {
        let ident = self.request_identity(req).await;
        match ident.identity() {
            Some(id) => match Uuid::parse_str(&id) {
                Ok(id) => {
                    audit.set_user_id(id);
                    Ok(id)
                }
                Err(_e) => Err(oauth2::ErrorResponse::invalid_request("user_id is invalid")),
            },
            None => Err(oauth2::ErrorResponse::invalid_request(
                "authentication required",
            )),
        }
    }

    /// Redirect using URI
    pub(crate) fn response_redirect(&self, uri: Url) -> oauth2::Result<actix_web::HttpResponse> {
        Ok(actix_web::HttpResponse::Found()
            .header("location", uri.as_str())
            .finish())
    }

    /// JSON response
    pub(crate) fn response_json_untyped(
        &self,
        data: String,
    ) -> HttpResult<actix_web::HttpResponse> {
        Ok(actix_web::HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .body(data))
    }

    /// JSON response
    pub(crate) fn response_json<T: serde::Serialize>(
        &self,
        data: Result<T>,
    ) -> HttpResult<Json<T>> {
        let data = data.map_err(HttpError::bad_request)?;
        Ok(Json(data))
    }

    /// Render template using default context
    pub(crate) fn response_template(
        &self,
        client: &Client,
        template: &str,
    ) -> oauth2::Result<actix_web::HttpResponse> {
        let output = client
            .template_html(&self.handlebars, template, self.template_context(client))
            .map_err(|e| oauth2::ErrorResponse::server_error(e))?;

        Ok(actix_web::HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(output))
    }

    /// Render template using context
    pub(crate) fn response_template_context<T: serde::Serialize>(
        &self,
        client: &Client,
        template: &str,
        context: T,
    ) -> oauth2::Result<actix_web::HttpResponse> {
        let output = client
            .template_html(&self.handlebars, template, context)
            .map_err(|e| oauth2::ErrorResponse::server_error(e))?;

        Ok(actix_web::HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(output))
    }

    pub(crate) fn mail_template<T: serde::Serialize>(
        &self,
        client: &Client,
        template: &str,
        context: T,
    ) -> oauth2::Result<String> {
        let output = client
            .template_mail_text(&self.handlebars, template, context)
            .map_err(|e| oauth2::ErrorResponse::server_error(e))?;

        Ok(output)
    }

    pub(crate) fn template_context(&self, client: &Client) -> Context {
        Context::build(&self.config.oauth2.providers, client)
    }

    pub(crate) async fn template_csrf_context(&self, client: &Client) -> oauth2::Result<Context> {
        self.csrf_token(client).await.map(|token| {
            let mut context = Context::build(&self.config.oauth2.providers, client);
            context.csrf_token(token);
            context
        })
    }

    pub(crate) async fn template_csrf_message_context(
        &self,
        client: &Client,
        message: Option<String>,
    ) -> oauth2::Result<Context> {
        self.csrf_token(client).await.map(|token| {
            let mut context = Context::build(&self.config.oauth2.providers, client);
            context.csrf_token(token);
            if let Some(message) = message {
                context.message(message);
            }
            context
        })
    }

    pub(crate) async fn template_csrf_error_context(
        &self,
        client: &Client,
        e: (i64, oauth2::ErrorResponse),
    ) -> oauth2::Result<Context> {
        self.csrf_token(client).await.map(|token| {
            let mut context = Context::build(&self.config.oauth2.providers, client);
            context.csrf_token(token);
            context.error(e);
            context
        })
    }

    pub(crate) fn template_error_context(
        &self,
        client: &Client,
        e: (i64, oauth2::ErrorResponse),
    ) -> Context {
        let mut context = Context::build(&self.config.oauth2.providers, client);
        context.error(e);
        context
    }
}

impl Client {
    pub(crate) fn template_html<T: serde::Serialize>(
        &self,
        handlebars: &handlebars::Handlebars,
        input: &str,
        context: T,
    ) -> Result<String> {
        let output = handlebars.render_template(input, &context)?;
        let wrapper_template = if !self.templates.html.content.is_empty() {
            &self.templates.html.content
        } else {
            TEMPLATE_HTML_WRAPPER
        };
        let output = handlebars.render_template(
            wrapper_template,
            &json!({
                "content": output,
            }),
        )?;
        Ok(output)
    }

    pub(crate) fn template_mail_text<T: serde::Serialize>(
        &self,
        handlebars: &handlebars::Handlebars,
        input: &str,
        context: T,
    ) -> Result<String> {
        let output = handlebars.render_template(input, &context)?;
        let wrapper_template = if !self.templates.mail_text.content.is_empty() {
            &self.templates.mail_text.content
        } else {
            TEMPLATE_MAIL_TEXT_WRAPPER
        };
        let output = handlebars.render_template(
            wrapper_template,
            &json!({
                "content": output,
            }),
        )?;
        Ok(output)
    }
}

impl HttpServer {
    pub(crate) async fn client_required(&self, auth: BasicAuth) -> HttpResult<Client> {
        let client_secret = match auth.secret() {
            Some(client_secret) => Ok(client_secret.to_string()),
            None => Err(HttpError::unauthorized("client_secret is required")),
        }?;
        self.client_from_secret(&client_secret)
            .await
            .map_err(HttpError::unauthorized)
    }

    fn client_from_config(&self, id: Uuid, config: &ConfigOauth2Client) -> oauth2::Result<Client> {
        if config.enable {
            Ok(Client {
                server_authorize_uri: self.uri_oauth2_authorize(),
                server_token_uri: self.uri_oauth2_token(),
                server_introspect_uri: self.uri_oauth2_introspect(),
                client_id: id,
                client_secret: config.secret.to_string(),
                redirect_uri: config.redirect_uri.clone(),
                client_name: config.name.to_string(),
                client_uri: config.uri.clone(),
                enable: config.enable,
                scope: oauth2::Scope::from_ref(&config.scope),
                user_scope: oauth2::Scope::from_ref(&config.user_scope),
                register_enable: config.register_enable,
                register_scope: oauth2::Scope::from_ref(&config.register_scope),
                ttl: config.ttl.clone(),
                templates: config.templates.clone(),
            })
        } else {
            Err(oauth2::ErrorResponse::unauthorized_client(
                "client is disabled",
            ))
        }
    }

    pub(crate) async fn client_secret_verify(
        &self,
        id: Uuid,
        client: &ConfigOauth2Client,
        secret: String,
    ) -> oauth2::Result<()> {
        let secret_check = self
            .postgres
            .secret_hash(&secret, &id.to_string())
            .await
            .map_err(|_e| oauth2::ErrorResponse::server_error("secret_hash error"))?;

        if client.secret == secret_check {
            Ok(())
        } else {
            Err(oauth2::ErrorResponse::unauthorized_client(
                "client_secret does not match",
            ))
        }
    }

    /// Verify user has access to client and requested scope
    ///
    /// If requested scope is empty, then access scope is returned
    pub(crate) async fn client_user_access_verify(
        &self,
        client: &Client,
        user_id: Uuid,
        scope: &oauth2::Scope,
    ) -> oauth2::Result<oauth2::Scope> {
        let access_scope = self
            .postgres
            .access_read(client, user_id)
            .await
            .map_err(|e| oauth2::ErrorResponse::access_denied(&e.to_string()))?;

        if access_scope.contains(scope) {
            if scope.is_empty() {
                Ok(access_scope)
            } else {
                Ok(scope.clone())
            }
        } else {
            Err(oauth2::ErrorResponse::access_denied("scope does not match"))
        }
    }

    pub(crate) async fn client_from_id(
        &self,
        audit: &mut Audit,
        id: &AuthClientId,
    ) -> oauth2::Result<Client> {
        let client = match self.config.oauth2.clients.get(&id.client_id) {
            Some(client) => {
                if id.redirect_uri == client.redirect_uri {
                    self.client_from_config(id.client_id, client)
                } else {
                    Err(oauth2::ErrorResponse::unauthorized_client(
                        "redirect_uri does not match",
                    ))
                }
            }
            None => Err(oauth2::ErrorResponse::unauthorized_client(
                "client_id not found",
            )),
        }?;
        audit.set_client(&client);
        Ok(client)
    }

    pub(crate) async fn client_from_id_or_code(
        &self,
        audit: &mut Audit,
        id: Option<AuthClientId>,
        code: Option<String>,
    ) -> oauth2::Result<Client> {
        match id {
            Some(id) => self.client_from_id(audit, &id).await,
            None => match code {
                Some(code) => self.client_from_code(audit, &code).await,
                None => Err(oauth2::ErrorResponse::unauthorized_client(
                    "client_id not found",
                )),
            },
        }
    }

    pub(crate) async fn client_from_secret(&self, client_secret: &str) -> oauth2::Result<Client> {
        let (id, secret) = self
            .postgres
            .key_secret_extract(client_secret)
            .map_err(oauth2::ErrorResponse::unauthorized_client)?;

        match self.config.oauth2.clients.get(&id) {
            Some(client) => {
                self.client_secret_verify(id, &client, secret).await?;
                self.client_from_config(id, client)
            }
            None => Err(oauth2::ErrorResponse::unauthorized_client(
                "client_id not found",
            )),
        }
    }

    pub(crate) async fn client_from_code(
        &self,
        audit: &mut Audit,
        code: &str,
    ) -> oauth2::Result<Client> {
        let id = self
            .postgres
            .code_read_client(code)
            .await
            .map_err(|e| oauth2::ErrorResponse::invalid_request(e))?;

        let client = match self.config.oauth2.clients.get(&id) {
            Some(client) => self.client_from_config(id, client),
            None => Err(oauth2::ErrorResponse::unauthorized_client(
                "client_id not found",
            )),
        }?;
        audit.set_client(&client);
        Ok(client)
    }

    pub(crate) async fn client_from_oauth2_csrf(
        &self,
        audit: &mut Audit,
        csrf: &str,
    ) -> oauth2::Result<Client> {
        let id = self
            .postgres
            .oauth2_code_read_client(csrf)
            .await
            .map_err(|e| oauth2::ErrorResponse::invalid_request(e))?;

        let client = match self.config.oauth2.clients.get(&id) {
            Some(client) => self.client_from_config(id, client),
            None => Err(oauth2::ErrorResponse::unauthorized_client(
                "client_id not found",
            )),
        }?;
        audit.set_client(&client);
        Ok(client)
    }
}

#[derive(Debug)]
pub enum LoginAction {
    Login,
    RequireUpdate,
}

impl HttpServer {
    pub(crate) async fn user_password_login(
        &self,
        audit: &mut Audit,
        args: UserLoginArgs,
    ) -> oauth2::Result<(String, LoginAction)> {
        let check = self
            .postgres
            .password_check(&args.email, &args.password)
            .await
            .map_err(oauth2::ErrorResponse::access_denied)?;

        if check.require_update {
            Ok((check.id.to_string(), LoginAction::RequireUpdate))
        } else {
            Ok((check.id.to_string(), LoginAction::Login))
        }
    }

    pub(crate) async fn user_password_reset_request(
        &self,
        audit: &mut Audit,
        client: &Client,
        email: String,
    ) -> oauth2::Result<()> {
        let code = self
            .postgres
            .code_insert_password_reset(client.client_id, client.ttl.code_s, &email)
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;

        let user_email = &email;
        let subject = "Password Reset Request";

        let uri = self.uri_auth_password_reset(&code);
        let mut uri_accept = uri.clone();
        uri_accept
            .query_pairs_mut()
            .append_pair("response_type", "accept");
        let mut uri_reject = uri.clone();
        uri_reject
            .query_pairs_mut()
            .append_pair("response_type", "reject");

        let text = self
            .mail_template(
                client,
                TEMPLATE_MAIL_PASSWORD_RESET,
                &json!({
                    "user_email": user_email,
                    "uri_accept": uri_accept,
                    "uri_reject": uri_reject,
                    "client_name": client.client_name,
                    "client_uri": client.client_uri,
                }),
            )
            .unwrap();

        self.mailto
            .send(self.mailto.build(user_email, subject, &text))
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;
        Ok(())
    }

    pub(crate) async fn user_password_reset_accept(
        &self,
        audit: &mut Audit,
        client: &Client,
        code: String,
        password: String,
    ) -> oauth2::Result<()> {
        let code = self
            .postgres
            .code_verify(client.client_id, &code, PostgresCodeTarget::PasswordReset)
            .await
            .map_err(|e| oauth2::ErrorResponse::access_denied(&e.to_string()))?;

        self.postgres
            .user_password_reset_accept(code.user_id.unwrap(), &password)
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;
        Ok(())
    }

    pub(crate) async fn user_password_reset_reject(
        &self,
        audit: &mut Audit,
        client: &Client,
        code: String,
    ) -> oauth2::Result<()> {
        self.postgres
            .code_verify(client.client_id, &code, PostgresCodeTarget::PasswordReset)
            .await
            .map_err(|e| oauth2::ErrorResponse::access_denied(&e.to_string()))?;
        Ok(())
    }

    pub(crate) async fn user_email_update(
        &self,
        id: Uuid,
        password: String,
        email_new: String,
    ) -> oauth2::Result<()> {
        self.postgres
            .user_email_update(id, &password, &email_new)
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;
        Ok(())
    }

    pub(crate) async fn user_password_update(
        &self,
        audit: &mut Audit,
        id: Uuid,
        password: String,
        password_new: String,
    ) -> oauth2::Result<()> {
        self.postgres
            .user_password_update(id, &password, &password_new)
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;
        Ok(())
    }

    pub(crate) async fn user_register_request(
        &self,
        audit: &mut Audit,
        client: &Client,
        email: String,
    ) -> oauth2::Result<()> {
        let code = self
            .postgres
            .code_insert_register(client.client_id, client.ttl.code_s, &email)
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;

        let user_email = &email;
        let subject = "Register Request";

        let uri = self.uri_auth_register(&code);
        let mut uri_accept = uri.clone();
        uri_accept
            .query_pairs_mut()
            .append_pair("response_type", "accept");
        let mut uri_reject = uri.clone();
        uri_reject
            .query_pairs_mut()
            .append_pair("response_type", "reject");

        let text = self
            .mail_template(
                client,
                TEMPLATE_MAIL_REGISTER,
                &json!({
                    "user_email": user_email,
                    "uri_accept": uri_accept,
                    "uri_reject": uri_reject,
                    "client_name": client.client_name,
                    "client_uri": client.client_uri,
                }),
            )
            .unwrap();

        self.mailto
            .send(self.mailto.build(user_email, subject, &text))
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;
        Ok(())
    }

    pub(crate) async fn user_register_accept_password(
        &self,
        audit: &mut Audit,
        client: &Client,
        code: String,
        args: UserRegisterAcceptArgs,
    ) -> oauth2::Result<String> {
        let code = self
            .postgres
            .code_verify(client.client_id, &code, PostgresCodeTarget::Register)
            .await
            .map_err(|e| oauth2::ErrorResponse::access_denied(&e.to_string()))?;

        let id = self
            .postgres
            .user_register_accept_password(
                &client,
                &client.register_scope,
                &code.email,
                &args.name,
                &args.password,
                args.password_allow_reset,
            )
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;
        Ok(id.to_string())
    }

    pub(crate) async fn user_register_reject(
        &self,
        audit: &mut Audit,
        client: &Client,
        code: String,
    ) -> oauth2::Result<()> {
        self.postgres
            .code_verify(client.client_id, &code, PostgresCodeTarget::Register)
            .await
            .map_err(|e| oauth2::ErrorResponse::access_denied(&e.to_string()))?;
        Ok(())
    }

    pub(crate) async fn user_delete_request(
        &self,
        audit: &mut Audit,
        client: &Client,
        id: Uuid,
        password: String,
    ) -> oauth2::Result<()> {
        let (code, email) = self
            .postgres
            .code_insert_delete(client.client_id, client.ttl.code_s, id, &password)
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;

        let user_email = &email;
        let subject = "Delete Request";

        let uri = self.uri_auth_delete(&code);
        let mut uri_accept = uri.clone();
        uri_accept
            .query_pairs_mut()
            .append_pair("response_type", "accept");
        let mut uri_reject = uri.clone();
        uri_reject
            .query_pairs_mut()
            .append_pair("response_type", "reject");

        let text = self
            .mail_template(
                client,
                TEMPLATE_MAIL_DELETE,
                &json!({
                    "user_email": user_email,
                    "uri_accept": uri_accept,
                    "uri_reject": uri_reject,
                    "client_name": client.client_name,
                    "client_uri": client.client_uri,
                }),
            )
            .unwrap();

        self.mailto
            .send(self.mailto.build(user_email, subject, &text))
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;
        Ok(())
    }

    pub(crate) async fn user_delete_accept(
        &self,
        audit: &mut Audit,
        client: &Client,
        id: Uuid,
        code: String,
    ) -> oauth2::Result<()> {
        let code = self
            .postgres
            .code_verify(client.client_id, &code, PostgresCodeTarget::Delete)
            .await
            .map_err(|e| oauth2::ErrorResponse::access_denied(&e.to_string()))?;

        let user_id = code.user_id.unwrap();
        if user_id != id {
            return Err(oauth2::ErrorResponse::invalid_request(
                "user_id does not match",
            ));
        }
        self.postgres
            .user_delete(user_id)
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;
        Ok(())
    }

    pub(crate) async fn user_delete_reject(
        &self,
        audit: &mut Audit,
        client: &Client,
        id: Uuid,
        code: String,
    ) -> oauth2::Result<()> {
        self.postgres
            .code_verify(client.client_id, &code, PostgresCodeTarget::Delete)
            .await
            .map_err(|e| oauth2::ErrorResponse::access_denied(&e.to_string()))?;
        Ok(())
    }
}

pub struct AuthClientId {
    pub client_id: Uuid,
    pub redirect_uri: Url,
    pub message: Option<String>,
}

impl AuthClientId {
    pub fn parse(query: RequestAuthQuery) -> oauth2::Result<Self> {
        let client_id = query.client_id;
        let redirect_uri = match Url::parse(&query.redirect_uri) {
            Ok(redirect_uri) => redirect_uri,
            Err(_e) => {
                return Err(oauth2::ErrorResponse::invalid_request(
                    "redirect_uri is invalid",
                ));
            }
        };
        let message = if let Some(message) = query.message.as_deref() {
            Some(message.to_string())
        } else {
            None
        };
        Ok(Self {
            client_id,
            redirect_uri,
            message,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BasicAuth(pub actix_web_httpauth::extractors::basic::BasicAuth);

impl BasicAuth {
    /// Client ID and secret are in urlencoded format
    /// [RFC](https://tools.ietf.org/html/rfc6749#section-2.3.1)
    pub fn id(&self) -> String {
        let id = self.0.user_id().borrow();
        percent_encoding::percent_decode_str(id)
            .decode_utf8()
            .unwrap()
            .to_string()
    }

    pub fn secret(&self) -> Option<String> {
        self.0.password().map(|x| {
            percent_encoding::percent_decode_str(x.borrow())
                .decode_utf8()
                .unwrap()
                .to_string()
        })
    }
}

impl paperclip_core::v2::schema::Apiv2Schema for BasicAuth {
    const NAME: Option<&'static str> = Some("basicAuth");

    fn security_scheme() -> Option<paperclip::v2::models::SecurityScheme> {
        let mut scheme = paperclip::v2::models::SecurityScheme::default();
        scheme.type_ = "basic".to_string();
        Some(scheme)
    }
}

impl paperclip::actix::OperationModifier for BasicAuth {}

impl Deref for BasicAuth {
    type Target = actix_web_httpauth::extractors::basic::BasicAuth;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl actix_web::FromRequest for BasicAuth {
    type Config = ();
    type Error = actix_web_httpauth::extractors::AuthenticationError<
        actix_web_httpauth::headers::www_authenticate::basic::Basic,
    >;
    type Future = futures::future::Ready<std::result::Result<Self, Self::Error>>;

    #[inline]
    fn from_request(req: &actix_web::HttpRequest, p: &mut actix_web::dev::Payload) -> Self::Future {
        match actix_web_httpauth::extractors::basic::BasicAuth::from_request(req, p).into_inner() {
            Ok(auth) => futures::future::ok(BasicAuth(auth)),
            Err(e) => futures::future::err(e),
        }
    }
}

impl fmt::Display for oauth2::ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl actix_http::ResponseError for oauth2::ErrorResponse {
    fn status_code(&self) -> http::StatusCode {
        http::StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> actix_http::Response {
        actix_http::Response::build(self.status_code())
            .content_type("text/html; charset=utf-8")
            .body(format!("{}", self))
    }
}

#[derive(Debug, Clone)]
pub enum Oauth2Redirect {
    Auth(oauth2::AuthorizationCodeRequest),
    Register,
}

impl HttpServer {
    pub(crate) async fn oauth2_provider_redirect_request(
        &self,
        audit: &mut Audit,
        client: &Client,
        provider: PostgresOauth2Provider,
        request: oauth2::AuthorizationCodeRequest,
    ) -> oauth2::Result<Url> {
        match provider {
            PostgresOauth2Provider::Sso => {
                if let Some(provider_client) = self.oauth2_providers.sso.as_ref() {
                    let (authorize_url, csrf_state) = provider_client
                        .authorize_url(::oauth2::CsrfToken::new_random)
                        .url();

                    self.postgres
                        .oauth2_code_insert_auth(
                            client.client_id,
                            client.ttl.oauth2_code_s,
                            provider,
                            csrf_state.secret(),
                            None,
                            request,
                        )
                        .await
                        .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;

                    return Ok(authorize_url);
                }
            }
            PostgresOauth2Provider::Microsoft => {
                if let Some(provider_client) = self.oauth2_providers.microsoft.as_ref() {
                    let (pkce_code_challenge, pkce_code_verifier) =
                        ::oauth2::PkceCodeChallenge::new_random_sha256();
                    let (authorize_url, csrf_state) = provider_client
                        .authorize_url(::oauth2::CsrfToken::new_random)
                        .add_scope(::oauth2::Scope::new(
                            "https://graph.microsoft.com/User.Read".to_string(),
                        ))
                        .set_pkce_challenge(pkce_code_challenge)
                        .url();

                    self.postgres
                        .oauth2_code_insert_auth(
                            client.client_id,
                            client.ttl.oauth2_code_s,
                            provider,
                            csrf_state.secret(),
                            Some(pkce_code_verifier.secret()),
                            request,
                        )
                        .await
                        .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;

                    return Ok(authorize_url);
                }
            }
        }
        Err(oauth2::ErrorResponse::invalid_request(
            "oauth2 provider not available",
        ))
    }

    pub(crate) async fn oauth2_provider_redirect_register_request(
        &self,
        audit: &mut Audit,
        client: &Client,
        code: String,
        provider: PostgresOauth2Provider,
    ) -> oauth2::Result<Url> {
        let code = self
            .postgres
            .code_verify(client.client_id, &code, PostgresCodeTarget::Register)
            .await
            .map_err(|e| oauth2::ErrorResponse::access_denied(&e.to_string()))?;

        match provider {
            PostgresOauth2Provider::Sso => {
                if let Some(provider_client) = self.oauth2_providers.sso.as_ref() {
                    let (authorize_url, csrf_state) = provider_client
                        .authorize_url(::oauth2::CsrfToken::new_random)
                        .url();

                    self.postgres
                        .oauth2_code_insert_register(
                            client.client_id,
                            client.ttl.oauth2_code_s,
                            provider,
                            csrf_state.secret(),
                            None,
                            &code.email,
                        )
                        .await
                        .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;

                    return Ok(authorize_url);
                }
            }
            PostgresOauth2Provider::Microsoft => {
                if let Some(provider_client) = self.oauth2_providers.microsoft.as_ref() {
                    let (pkce_code_challenge, pkce_code_verifier) =
                        ::oauth2::PkceCodeChallenge::new_random_sha256();
                    let (authorize_url, csrf_state) = provider_client
                        .authorize_url(::oauth2::CsrfToken::new_random)
                        .add_scope(::oauth2::Scope::new(
                            "https://graph.microsoft.com/User.Read".to_string(),
                        ))
                        .set_pkce_challenge(pkce_code_challenge)
                        .url();

                    self.postgres
                        .oauth2_code_insert_register(
                            client.client_id,
                            client.ttl.oauth2_code_s,
                            provider,
                            csrf_state.secret(),
                            Some(pkce_code_verifier.secret()),
                            &code.email,
                        )
                        .await
                        .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;

                    return Ok(authorize_url);
                }
            }
        }
        Err(oauth2::ErrorResponse::invalid_request(
            "oauth2 provider not available",
        ))
    }

    pub(crate) async fn oauth2_provider_microsoft_token_decode(
        &self,
        token: &str,
    ) -> oauth2::Result<(String, String)> {
        #[derive(Debug, Deserialize)]
        struct Response {
            sub: String,
            name: String,
        }

        let config = self.config.oauth2.providers.microsoft.as_ref().unwrap();
        match config.oidc_userinfo_uri.as_ref() {
            Some(oidc_userinfo_uri) => {
                let authorisation = format!("Bearer {}", token);
                let res = self
                    .client
                    .get(oidc_userinfo_uri.clone())
                    .header(http::header::AUTHORIZATION, authorisation)
                    .send()
                    .await
                    .map_err(Error::ReqwestError)
                    .map_err(oauth2::ErrorResponse::server_error)?
                    .error_for_status()
                    .map_err(Error::ReqwestError)
                    .map_err(oauth2::ErrorResponse::server_error)?
                    .json::<Response>()
                    .await
                    .map_err(Error::ReqwestError)
                    .map_err(oauth2::ErrorResponse::server_error)?;

                Ok((res.sub, res.name))
            }
            None => Err(oauth2::ErrorResponse::server_error(
                "oidc_userinfo_uri is required",
            )),
        }
    }

    pub(crate) async fn oauth2_provider_sso_token_introspect(
        &self,
        token: &str,
    ) -> oauth2::Result<(String, String)> {
        #[derive(Debug, Deserialize)]
        struct Response {
            sub: String,
            username: String,
        }

        let config = self.config.oauth2.providers.sso.as_ref().unwrap();
        match config.introspect_uri.as_ref() {
            Some(introspect_uri) => {
                let res = self
                    .client
                    .post(introspect_uri.clone())
                    .basic_auth(
                        config.client_id.to_string(),
                        Some(config.client_secret.to_string()),
                    )
                    .json(&json!({ "token": token }))
                    .send()
                    .await
                    .map_err(Error::ReqwestError)
                    .map_err(oauth2::ErrorResponse::server_error)?
                    .error_for_status()
                    .map_err(Error::ReqwestError)
                    .map_err(oauth2::ErrorResponse::server_error)?
                    .json::<Response>()
                    .await
                    .map_err(Error::ReqwestError)
                    .map_err(oauth2::ErrorResponse::server_error)?;

                Ok((res.sub, res.username))
            }
            None => Err(oauth2::ErrorResponse::server_error(
                "introspect_uri is required",
            )),
        }
    }

    pub(crate) async fn oauth2_provider_redirect_response(
        &self,
        audit: &mut Audit,
        client: &Client,
        request: RequestOauth2RedirectQuery,
    ) -> oauth2::Result<(String, Oauth2Redirect)> {
        use ::oauth2::{AsyncCodeTokenRequest, TokenResponse};

        let code = self
            .postgres
            .oauth2_code_verify(client.client_id, &request.state)
            .await
            .map_err(|e| oauth2::ErrorResponse::access_denied(&e.to_string()))?;

        match code.provider {
            PostgresOauth2Provider::Sso => {
                if let Some(provider_client) = self.oauth2_providers.sso.as_ref() {
                    let token = provider_client
                        .exchange_code(::oauth2::AuthorizationCode::new(request.code))
                        .request_async(::oauth2::reqwest::async_http_client)
                        .await
                        .map_err(|e| oauth2::ErrorResponse::access_denied(e.to_string()))?;

                    let (sub, name) = self
                        .oauth2_provider_sso_token_introspect(token.access_token().secret())
                        .await?;

                    match code.target {
                        PostgresOauth2Target::Auth => {
                            let user_id = self
                                .postgres
                                .user_oauth2_provider_check(code.provider, &sub)
                                .await
                                .map_err(|e| oauth2::ErrorResponse::access_denied(e.to_string()))?;

                            return Ok((
                                user_id.to_string(),
                                Oauth2Redirect::Auth(oauth2::AuthorizationCodeRequest::new(
                                    &client.client_id.to_string(),
                                    code.redirect_uri.unwrap(),
                                    &code.state,
                                    code.scope,
                                )),
                            ));
                        }
                        PostgresOauth2Target::Register => {
                            let user_id = self
                                .postgres
                                .user_register_accept_oauth2_provider(
                                    &client,
                                    &client.register_scope,
                                    &code.email,
                                    &name,
                                    code.provider,
                                    &sub,
                                )
                                .await
                                .map_err(|e| oauth2::ErrorResponse::server_error(e.to_string()))?;

                            return Ok((user_id.to_string(), Oauth2Redirect::Register));
                        }
                    }
                }
            }
            PostgresOauth2Provider::Microsoft => {
                if let Some(provider_client) = self.oauth2_providers.microsoft.as_ref() {
                    let pkce_verifier = ::oauth2::PkceCodeVerifier::new(code.pkce);

                    let token = provider_client
                        .exchange_code(::oauth2::AuthorizationCode::new(request.code))
                        .set_pkce_verifier(pkce_verifier)
                        .request_async(::oauth2::reqwest::async_http_client)
                        .await
                        .map_err(|e| oauth2::ErrorResponse::access_denied(e.to_string()))?;

                    let (sub, name) = self
                        .oauth2_provider_microsoft_token_decode(token.access_token().secret())
                        .await?;

                    match code.target {
                        PostgresOauth2Target::Auth => {
                            let user_id = self
                                .postgres
                                .user_oauth2_provider_check(code.provider, &sub)
                                .await
                                .map_err(|e| oauth2::ErrorResponse::access_denied(e.to_string()))?;

                            return Ok((
                                user_id.to_string(),
                                Oauth2Redirect::Auth(oauth2::AuthorizationCodeRequest::new(
                                    &client.client_id.to_string(),
                                    code.redirect_uri.unwrap(),
                                    &code.state,
                                    code.scope,
                                )),
                            ));
                        }
                        PostgresOauth2Target::Register => {
                            let user_id = self
                                .postgres
                                .user_register_accept_oauth2_provider(
                                    &client,
                                    &client.register_scope,
                                    &code.email,
                                    &name,
                                    code.provider,
                                    &sub,
                                )
                                .await
                                .map_err(|e| oauth2::ErrorResponse::server_error(e.to_string()))?;

                            return Ok((user_id.to_string(), Oauth2Redirect::Register));
                        }
                    }
                }
            }
        }
        Err(oauth2::ErrorResponse::invalid_request(
            "oauth2 provider not available",
        ))
    }

    pub(crate) async fn oauth2_authorization_code(
        &self,
        audit: &mut Audit,
        client: &Client,
        request: oauth2::AuthorizationCodeRequest,
        user_id: String,
    ) -> oauth2::Result<Url> {
        match Uuid::parse_str(&user_id) {
            Ok(user_id) => {
                let scope = self
                    .client_user_access_verify(client, user_id, request.scope())
                    .await?;

                let code = self
                    .postgres
                    .code_insert_auth(
                        client,
                        client.ttl.code_s,
                        user_id,
                        request.state(),
                        &scope.to_string(),
                    )
                    .await
                    .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;

                let args = oauth2::AuthorizationCodeResponseArgs::new(&code);
                let (_, redirect_uri) =
                    self.oauth2_authorization_code_response(client, request, args)?;
                Ok(redirect_uri)
            }
            Err(_e) => Err(oauth2::ErrorResponse::invalid_request("user_id is invalid")),
        }
    }

    pub(crate) async fn oauth2_access_token(
        &self,
        client: &Client,
        request: oauth2::AccessTokenRequest,
    ) -> oauth2::Result<oauth2::AccessTokenResponse> {
        let code = self
            .postgres
            .code_verify(client.client_id, request.code(), PostgresCodeTarget::Auth)
            .await
            .map_err(|e| oauth2::ErrorResponse::access_denied(&e.to_string()))?;

        let token = self
            .postgres
            .token_insert(
                client,
                request.client_secret(),
                code.user_id.unwrap(),
                client.ttl.token_access_s,
                "oauth2_access_token",
                &code.scope,
            )
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;

        let args = oauth2::TokenResponseArgs::new(
            &token.access_token,
            client.ttl.token_access_s,
            token.scope,
            Some(&token.refresh_token),
        );
        Ok(self.oauth2_access_token_response(client, request, args))
    }

    pub(crate) async fn oauth2_refresh_token(
        &self,
        client: &Client,
        request: oauth2::RefreshTokenRequest,
    ) -> oauth2::Result<oauth2::AccessTokenResponse> {
        let token = self
            .postgres
            .token_refresh(
                client.client_id,
                client.ttl.token_access_s,
                client.ttl.token_refresh_s,
                request.client_secret(),
                request.refresh_token(),
            )
            .await
            .map_err(|e| oauth2::ErrorResponse::invalid_request(&e.to_string()))?;

        let args = oauth2::TokenResponseArgs::new(
            &token.access_token,
            client.ttl.token_access_s,
            token.scope,
            Some(&token.refresh_token),
        );
        Ok(self.oauth2_refresh_token_response(client, request, args))
    }

    pub(crate) async fn oauth2_introspection(
        &self,
        client: &Client,
        request: oauth2::IntrospectionRequest,
    ) -> oauth2::Result<Option<oauth2::IntrospectionResponse>> {
        let token = self
            .postgres
            .token_introspect(
                client.client_id,
                client.ttl.token_refresh_s,
                request.client_secret(),
                request.token(),
            )
            .await
            .map_err(|e| oauth2::ErrorResponse::server_error(&e.to_string()))?;

        match token {
            Some(args) => Ok(Some(
                self.oauth2_introspection_response(client, request, args),
            )),
            None => Ok(None),
        }
    }
}
