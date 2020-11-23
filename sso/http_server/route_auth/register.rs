use crate::http_server::internal::*;

#[api_v2_operation(summary = "User registration interface")]
pub async fn get(
    server: Data<HttpServer>,
    req: HttpRequest,
    query: Query<RequestAuthResponseQuery>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_auth_register", &req);
        let query = server_oauth2_validate!(&server, query);
        let query = AuthRegisterQueryParse::parse(query)?;

        let client = server.client_from_code(&mut audit, query.code()).await?;

        server_oauth2_error!(&server, audit, &client, TEMPLATE_ERROR, async {
            let context = server.template_csrf_context(&client).await?;

            let template = query.template_get();

            server.response_template_context(&client, template, context)
        })
    })
}

#[api_v2_operation(summary = "User registration interface")]
pub async fn post(
    server: Data<HttpServer>,
    req: HttpRequest,
    query: Query<RequestAuthResponseQuery>,
    body: Form<RequestAuthRegister>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_auth_register", &req);
        let query = server_oauth2_validate!(&server, query);
        let query = AuthRegisterQueryParse::parse(query)?;
        let body = server_oauth2_validate!(&server, body);
        let csrf_token = body.csrf_token.clone();

        let client = server.client_from_code(&mut audit, query.code()).await?;

        server.csrf_verify(&client, csrf_token).await?;

        let template = query.template_get();

        server_oauth2_form_error!(&server, audit, &client, &template, async {
            match query {
                AuthRegisterQueryParse::Accept(code) => {
                    let request = AuthRegisterFormParse::parse(body)?;

                    match request {
                        AuthRegisterFormParse::Password(args) => {
                            let user_id = server.request_identity(&req).await;

                            let id = server
                                .user_register_accept_password(&mut audit, &client, code, args)
                                .await?;
                            user_id.remember(id);

                            server.response_template(&client, TEMPLATE_AUTH_REGISTER_ACCEPT_OK)
                        }
                        AuthRegisterFormParse::Oauth2(args) => {
                            let redirect_uri = server
                                .oauth2_provider_redirect_register_request(
                                    &mut audit, &client, code, args,
                                )
                                .await?;

                            Ok(server.response_redirect(redirect_uri))
                        }
                    }
                }
                AuthRegisterQueryParse::Reject(code) => {
                    server
                        .user_register_reject(&mut audit, &client, code)
                        .await?;

                    server.response_template(&client, TEMPLATE_AUTH_REGISTER_REJECT_OK)
                }
            }
        })
    })
}

enum AuthRegisterQueryParse {
    Accept(String),
    Reject(String),
}

enum AuthRegisterFormParse {
    Password(UserRegisterAcceptArgs),
    Oauth2(PostgresOauth2Provider),
}

impl AuthRegisterQueryParse {
    fn template_get(&self) -> &'static str {
        match self {
            Self::Accept(_) => TEMPLATE_AUTH_REGISTER_ACCEPT,
            Self::Reject(_) => TEMPLATE_AUTH_REGISTER_REJECT,
        }
    }

    fn code(&self) -> &str {
        match self {
            Self::Accept(code) => code,
            Self::Reject(code) => code,
        }
    }

    fn parse(req: RequestAuthResponseQuery) -> oauth2::Result<Self> {
        match req.response_type.as_ref() {
            "accept" => Ok(Self::Accept(req.code)),
            "reject" => Ok(Self::Reject(req.code)),
            _ => Err(oauth2::ErrorResponse::invalid_request(
                "response_type is invalid",
            )),
        }
    }
}

impl AuthRegisterFormParse {
    fn parse(req: RequestAuthRegister) -> oauth2::Result<Self> {
        match req.register_type.as_ref() {
            "password" => {
                let name = if let Some(name) = req.name.as_deref() {
                    name.to_string()
                } else {
                    return Err(oauth2::ErrorResponse::invalid_request("name is required"));
                };
                let password = if let Some(password) = req.password.as_deref() {
                    password.to_string()
                } else {
                    return Err(oauth2::ErrorResponse::invalid_request(
                        "password is required",
                    ));
                };
                let password_confirm =
                    if let Some(password_confirm) = req.password_confirm.as_deref() {
                        password_confirm.to_string()
                    } else {
                        return Err(oauth2::ErrorResponse::invalid_request(
                            "password_confirm is required",
                        ));
                    };
                if password != password_confirm {
                    return Err(oauth2::ErrorResponse::invalid_request(
                        "password does not match password_confirm",
                    ));
                }
                let password_allow_reset =
                    if let Some(password_allow_reset) = req.password_allow_reset.as_deref() {
                        matches!(password_allow_reset, "true")
                    } else {
                        false
                    };
                Ok(Self::Password(UserRegisterAcceptArgs {
                    name,
                    password,
                    password_allow_reset,
                }))
            }
            "oauth2" => {
                let oauth2_provider = if let Some(oauth2_provider) = req.oauth2_provider.as_deref()
                {
                    PostgresOauth2Provider::from_str(oauth2_provider)?
                } else {
                    return Err(oauth2::ErrorResponse::invalid_request(
                        "oauth2_provider is required",
                    ));
                };
                Ok(Self::Oauth2(oauth2_provider))
            }
            _ => Err(oauth2::ErrorResponse::invalid_request(
                "register_type is invalid",
            )),
        }
    }
}
