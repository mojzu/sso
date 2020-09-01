use crate::server::{internal::*, template};

#[api_v2_operation(summary = "User reset password interface")]
pub async fn get(
    server: Data<Server>,
    req: HttpRequest,
    query: Query<RequestAuthResponseQuery>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_auth_password_reset", &req);
        let query = server_oauth2_validate!(&server, query);
        let query = AuthPasswordResetParse::parse_query(query)?;

        let client = server.client_from_code(&mut audit, query.code()).await?;

        server_oauth2_error!(&server, audit, &client, template::ERROR, async {
            let context = server.template_csrf_context(&client).await?;

            let template = query.template_get();

            server.response_template_context(&client, template, context)
        })
    })
}

#[api_v2_operation(summary = "User reset password interface")]
pub async fn post(
    server: Data<Server>,
    req: HttpRequest,
    query: Query<RequestAuthResponseQuery>,
    body: Form<RequestAuthPasswordReset>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_auth_password_reset", &req);
        let query = server_oauth2_validate!(&server, query);
        let query = AuthPasswordResetParse::parse_query(query)?;
        let body = server_oauth2_validate!(&server, body);
        let csrf_token = body.csrf_token.clone();

        let client = server.client_from_code(&mut audit, query.code()).await?;

        server.csrf_verify(&client, csrf_token).await?;

        let template = query.template_get();

        server_oauth2_form_error!(&server, audit, &client, &template, async {
            match query {
                AuthPasswordResetParse::Accept(code) => {
                    let password = AuthPasswordResetParse::parse_request(body)?;
                    server
                        .user_password_reset_accept(&mut audit, &client, code, password)
                        .await?;

                    server.response_template(&client, template::AUTH_PASSWORD_RESET_ACCEPT_OK)
                }
                AuthPasswordResetParse::Reject(code) => {
                    server
                        .user_password_reset_reject(&mut audit, &client, code)
                        .await?;

                    server.response_template(&client, template::AUTH_PASSWORD_RESET_REJECT_OK)
                }
            }
        })
    })
}

enum AuthPasswordResetParse {
    Accept(String),
    Reject(String),
}

impl AuthPasswordResetParse {
    fn template_get(&self) -> &'static str {
        match self {
            Self::Accept(_) => template::AUTH_PASSWORD_RESET_ACCEPT,
            Self::Reject(_) => template::AUTH_PASSWORD_RESET_REJECT,
        }
    }

    fn code(&self) -> &str {
        match self {
            Self::Accept(code) => code,
            Self::Reject(code) => code,
        }
    }

    fn parse_query(req: RequestAuthResponseQuery) -> oauth2::Result<Self> {
        match req.response_type.as_ref() {
            "accept" => Ok(Self::Accept(req.code)),
            "reject" => Ok(Self::Reject(req.code)),
            _ => Err(oauth2::ErrorResponse::invalid_request(
                "response_type is invalid",
            )),
        }
    }

    fn parse_request(req: RequestAuthPasswordReset) -> oauth2::Result<String> {
        let password_new = if let Some(password_new) = req.password_new {
            password_new
        } else {
            return Err(oauth2::ErrorResponse::invalid_request(
                "password_new is required",
            ));
        };
        let password_confirm = if let Some(password_confirm) = req.password_confirm {
            password_confirm
        } else {
            return Err(oauth2::ErrorResponse::invalid_request(
                "password_confirm is required",
            ));
        };
        if password_new != password_confirm {
            return Err(oauth2::ErrorResponse::invalid_request(
                "password_new does not match password_confirm",
            ));
        }
        Ok(password_new)
    }
}
