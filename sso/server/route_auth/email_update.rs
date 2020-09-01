use crate::server::{internal::*, template};

#[api_v2_operation(summary = "User update email interface")]
pub async fn get(
    server: Data<Server>,
    req: HttpRequest,
    query: Query<RequestAuthQuery>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_auth_email_update", &req);
        let query = server_oauth2_validate!(&server, query);
        let query = AuthClientId::parse(query)?;

        let client = server.client_from_id(&mut audit, &query).await?;

        server_oauth2_error!(&server, audit, &client, template::ERROR, async {
            let _id = server.request_identity_required(&mut audit, &req).await?;

            let context = server.template_csrf_context(&client).await?;

            server.response_template_context(&client, template::AUTH_EMAIL_UPDATE, context)
        })
    })
}

#[api_v2_operation(summary = "User update email interface")]
pub async fn post(
    server: Data<Server>,
    req: HttpRequest,
    query: Query<RequestAuthQuery>,
    body: Form<RequestAuthEmailUpdate>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_auth_email_update", &req);
        let query = server_oauth2_validate!(&server, query);
        let query = AuthClientId::parse(query)?;
        let body = server_oauth2_validate!(&server, body);
        let (csrf_token, password, email_new) = body.into_inner()?;

        let client = server.client_from_id(&mut audit, &query).await?;

        server.csrf_verify(&client, csrf_token).await?;

        server_oauth2_form_error!(
            &server,
            audit,
            &client,
            template::AUTH_EMAIL_UPDATE,
            async {
                let user_id = server.request_identity_required(&mut audit, &req).await?;

                server
                    .user_email_update(user_id, password, email_new)
                    .await?;

                server.response_template(&client, template::AUTH_EMAIL_UPDATE_OK)
            }
        )
    })
}

impl RequestAuthEmailUpdate {
    fn into_inner(self) -> oauth2::Result<(String, String, String)> {
        if self.email_new != self.email_confirm {
            return Err(oauth2::ErrorResponse::invalid_request(
                "email_new does not match email_confirm",
            ));
        }
        Ok((self.csrf_token, self.password, self.email_new))
    }
}
