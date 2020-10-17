use crate::http_server::internal::*;

#[api_v2_operation(summary = "OAuth2 authorization endpoint")]
pub async fn get(
    server: Data<HttpServer>,
    req: HttpRequest,
    query: Query<RequestOauth2AuthorizeQuery>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_oauth2_authorize", &req);
        let query = server_oauth2_validate!(&server, query);

        let request = server.oauth2_authorize_parse_request(
            Some(&query.response_type),
            Some(&query.client_id),
            Some(&query.redirect_uri),
            Some(&query.state),
            query.scope.as_deref(),
        )?;

        let client_id: AuthClientId = (request.client_id(), request.redirect_uri()).try_into()?;
        let client = server.client_from_id(&mut audit, &client_id).await?;

        server_oauth2_error!(&server, audit, &client, TEMPLATE_ERROR, async {
            let ident = server.request_identity(&req).await;

            if let Some(id) = ident.identity() {
                let redirect_uri = server
                    .oauth2_authorization_code(&mut audit, &client, request, id)
                    .await?;

                Ok(server.response_redirect(redirect_uri))
            } else {
                let context = server.template_csrf_context(&client).await?;

                server.response_template_context(&client, TEMPLATE_AUTH, context)
            }
        })
    })
}

#[api_v2_operation(summary = "OAuth2 authorization endpoint")]
pub async fn post(
    server: Data<HttpServer>,
    req: HttpRequest,
    query: Query<RequestOauth2AuthorizeQuery>,
    body: Form<RequestOauth2Authorize>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_oauth2_authorize", &req);
        let query = server_oauth2_validate!(&server, query);
        let body = server_oauth2_validate!(&server, body);

        let oauth2_request = server.oauth2_authorize_parse_request(
            Some(&query.response_type),
            Some(&query.client_id),
            Some(&query.redirect_uri),
            Some(&query.state),
            query.scope.as_deref(),
        )?;
        let (csrf_token, request) = AuthorizeFormParse::parse(body)?;

        let client_id: AuthClientId =
            (oauth2_request.client_id(), oauth2_request.redirect_uri()).try_into()?;
        let client = server.client_from_id(&mut audit, &client_id).await?;

        server.csrf_verify(&client, csrf_token).await?;

        server_oauth2_form_error!(&server, audit, &client, TEMPLATE_AUTH, async {
            match request {
                AuthorizeFormParse::PasswordLogin(request) => {
                    let ident = server.request_identity(&req).await;
                    let (user_id, action) = server.user_password_login(&mut audit, request).await?;
                    ident.remember(user_id);

                    let redirect_uri = match action {
                        LoginAction::Login => {
                            oauth2_request.user_redirect_uri(client.server_authorize_uri())
                        }
                        LoginAction::RequireUpdate => server
                            .uri_auth_password_update(&client, Some("Password update is required")),
                    };

                    Ok(server.response_redirect(redirect_uri))
                }
                AuthorizeFormParse::PasswordReset(email) => {
                    server
                        .user_password_reset_request(&mut audit, &client, email)
                        .await?;

                    server.response_template(&client, TEMPLATE_AUTH_PASSWORD_RESET)
                }
                AuthorizeFormParse::Oauth2(provider) => {
                    let redirect_uri = server
                        .oauth2_provider_redirect_request(
                            &mut audit,
                            &client,
                            provider,
                            oauth2_request,
                        )
                        .await?;

                    Ok(server.response_redirect(redirect_uri))
                }
                AuthorizeFormParse::Register(email) => {
                    server
                        .user_register_request(&mut audit, &client, email)
                        .await?;

                    server.response_template(&client, TEMPLATE_AUTH_REGISTER)
                }
            }
        })
    })
}

enum AuthorizeFormParse {
    PasswordLogin(UserLoginArgs),
    PasswordReset(String),
    Oauth2(PostgresOauth2Provider),
    Register(String),
}

impl AuthorizeFormParse {
    fn parse(req: RequestOauth2Authorize) -> oauth2::Result<(String, Self)> {
        match req.auth_type.as_ref() {
            "password_login" => {
                let email = if let Some(email) = req.email.as_deref() {
                    email.to_string()
                } else {
                    return Err(oauth2::ErrorResponse::invalid_request("email is required"));
                };
                let password = if let Some(password) = req.password.as_deref() {
                    password.to_string()
                } else {
                    return Err(oauth2::ErrorResponse::invalid_request(
                        "password is required",
                    ));
                };
                Ok((
                    req.csrf_token,
                    Self::PasswordLogin(UserLoginArgs { email, password }),
                ))
            }
            "password_reset" => {
                let email = if let Some(email) = req.email.as_deref() {
                    email.to_string()
                } else {
                    return Err(oauth2::ErrorResponse::invalid_request("email is required"));
                };
                Ok((req.csrf_token, Self::PasswordReset(email)))
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
                Ok((req.csrf_token, Self::Oauth2(oauth2_provider)))
            }
            "register" => {
                let email = if let Some(email) = req.email.as_deref() {
                    email.to_string()
                } else {
                    return Err(oauth2::ErrorResponse::invalid_request("email is required"));
                };
                Ok((req.csrf_token, Self::Register(email)))
            }
            _ => Err(oauth2::ErrorResponse::invalid_request(
                "auth_type is invalid",
            )),
        }
    }
}
