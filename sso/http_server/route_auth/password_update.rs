use crate::http_server::internal::*;

#[api_v2_operation(summary = "User update password interface")]
pub async fn get(
    server: Data<HttpServer>,
    req: HttpRequest,
    query: Query<RequestAuthQuery>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_auth_password_update", &req);
        let query = server_oauth2_validate!(&server, query);
        let query = AuthClientId::parse(query)?;

        let client = server.client_from_id(&mut audit, &query).await?;

        server_oauth2_error!(&server, audit, &client, TEMPLATE_ERROR, async {
            let _id = server.request_identity_required(&mut audit, &req).await?;

            let context = server
                .template_csrf_message_context(&client, query.message)
                .await?;

            server.response_template_context(&client, TEMPLATE_AUTH_PASSWORD_UPDATE, context)
        })
    })
}

#[api_v2_operation(summary = "User update password interface")]
pub async fn post(
    server: Data<HttpServer>,
    req: HttpRequest,
    query: Query<RequestAuthQuery>,
    body: Form<RequestAuthPasswordUpdate>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_auth_password_update", &req);
        let query = server_oauth2_validate!(&server, query);
        let query = AuthClientId::parse(query)?;
        let body = server_oauth2_validate!(&server, body);
        let (csrf_token, password, password_new) = body.into_inner()?;

        let client = server.client_from_id(&mut audit, &query).await?;

        server.csrf_verify(&client, csrf_token).await?;

        server_oauth2_form_error!(
            &server,
            audit,
            &client,
            TEMPLATE_AUTH_PASSWORD_UPDATE,
            async {
                let user_id = server.request_identity_required(&mut audit, &req).await?;

                server
                    .user_password_update(&mut audit, user_id, password, password_new)
                    .await?;

                server.response_template(&client, TEMPLATE_AUTH_PASSWORD_UPDATE_OK)
            }
        )
    })
}

impl TryFrom<(&str, &Url)> for AuthClientId {
    type Error = oauth2::ErrorResponse;

    fn try_from(x: (&str, &Url)) -> oauth2::Result<Self> {
        match Uuid::parse_str(x.0) {
            Ok(client_id) => Ok(Self {
                client_id,
                redirect_uri: x.1.clone(),
                message: None,
            }),
            Err(_e) => Err(oauth2::ErrorResponse::invalid_request(
                "client_id is invalid",
            )),
        }
    }
}

impl RequestAuthPasswordUpdate {
    fn into_inner(self) -> oauth2::Result<(String, String, String)> {
        if self.password_new != self.password_confirm {
            return Err(oauth2::ErrorResponse::invalid_request(
                "password_new does not match password_confirm",
            ));
        }
        Ok((self.csrf_token, self.password, self.password_new))
    }
}
