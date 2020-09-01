use crate::server::{internal::*, template};

#[api_v2_operation(summary = "Delete user interface")]
pub async fn get(
    server: Data<Server>,
    req: HttpRequest,
    query: Query<RequestAuthDeleteQuery>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_auth_delete", &req);
        let query = server_oauth2_validate!(&server, query);

        let (query_client, query_code, query) = AuthDeleteQueryParse::parse(&query)?;

        let client = server
            .client_from_id_or_code(&mut audit, query_client, query_code)
            .await?;

        server_oauth2_error!(&server, audit, &client, template::ERROR, async {
            let _id = server.request_identity_required(&mut audit, &req).await?;

            let context = server.template_csrf_context(&client).await?;

            let template = query.template_get();

            server.response_template_context(&client, template, context)
        })
    })
}

#[api_v2_operation(summary = "Delete user interface")]
pub async fn post(
    server: Data<Server>,
    req: HttpRequest,
    query: Query<RequestAuthDeleteQuery>,
    body: Form<RequestAuthDelete>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_auth_delete", &req);
        let query = server_oauth2_validate!(&server, query);
        let body = server_oauth2_validate!(&server, body);

        let (query_client, query_code, query) = AuthDeleteQueryParse::parse(&query)?;
        let (csrf_token, password) = (body.csrf_token, body.password);

        let client = server
            .client_from_id_or_code(&mut audit, query_client, query_code)
            .await?;

        server.csrf_verify(&client, csrf_token).await?;

        let template = query.template_get();

        server_oauth2_form_error!(&server, audit, &client, &template, async {
            let id = server.request_identity_required(&mut audit, &req).await?;

            let body = match query {
                AuthDeleteQueryParse::Request => {
                    server
                        .user_delete_request(&mut audit, &client, id, password)
                        .await?;

                    template::AUTH_DELETE_OK
                }
                AuthDeleteQueryParse::Accept(code) => {
                    server
                        .user_delete_accept(&mut audit, &client, id, code)
                        .await?;

                    let ident = server.request_identity(&req).await;
                    ident.forget();

                    template::AUTH_DELETE_ACCEPT_OK
                }
                AuthDeleteQueryParse::Reject(code) => {
                    server
                        .user_delete_reject(&mut audit, &client, id, code)
                        .await?;

                    template::AUTH_DELETE_REJECT_OK
                }
            };

            server.response_template(&client, body)
        })
    })
}

enum AuthDeleteQueryParse {
    Request,
    Accept(String),
    Reject(String),
}

impl AuthDeleteQueryParse {
    fn template_get(&self) -> &'static str {
        match self {
            Self::Request => template::AUTH_DELETE,
            Self::Accept(_) => template::AUTH_DELETE_ACCEPT,
            Self::Reject(_) => template::AUTH_DELETE_REJECT,
        }
    }

    fn parse(
        query: &RequestAuthDeleteQuery,
    ) -> oauth2::Result<(Option<AuthClientId>, Option<String>, Self)> {
        let client_id = if let Some(client_id) = query.client_id {
            let redirect_uri = if let Some(redirect_uri) = query.redirect_uri.as_deref() {
                match Url::parse(redirect_uri) {
                    Ok(redirect_uri) => redirect_uri,
                    Err(_e) => {
                        return Err(oauth2::ErrorResponse::invalid_request(
                            "redirect_uri is invalid",
                        ));
                    }
                }
            } else {
                return Err(oauth2::ErrorResponse::invalid_request(
                    "redirect_uri is required",
                ));
            };
            Some(AuthClientId {
                client_id,
                redirect_uri,
                message: None,
            })
        } else {
            None
        };

        let response_type = if let Some(response_type) = query.response_type.as_deref() {
            response_type
        } else {
            return Ok((client_id, None, Self::Request));
        };
        let code = if let Some(code) = query.code.as_deref() {
            code.to_string()
        } else {
            return Err(oauth2::ErrorResponse::invalid_request("code is required"));
        };

        match response_type {
            "accept" => Ok((client_id, Some(code.clone()), Self::Accept(code))),
            "reject" => Ok((client_id, Some(code.clone()), Self::Reject(code))),
            _ => Err(oauth2::ErrorResponse::invalid_request(
                "response_type is invalid",
            )),
        }
    }
}
