use crate::http_server::internal::*;

#[api_v2_operation(summary = "OAuth2 redirect endpoint")]
pub async fn get(
    server: Data<HttpServer>,
    req: HttpRequest,
    query: Query<RequestOauth2RedirectQuery>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_oauth2_redirect", &req);
        let query = server_oauth2_validate!(&server, query);

        let client = server
            .client_from_oauth2_csrf(&mut audit, &query.state)
            .await?;

        server_oauth2_error!(&server, audit, &client, TEMPLATE_ERROR, async {
            let user_id = server.request_identity(&req).await;

            let (id, redirect_request) = server
                .oauth2_provider_redirect_response(&mut audit, &client, query)
                .await?;
            user_id.remember(id.clone());

            match redirect_request {
                Oauth2Redirect::Auth(oauth2_request) => {
                    let redirect_uri = server
                        .oauth2_authorization_code(&mut audit, &client, oauth2_request, id)
                        .await?;

                    Ok(server.response_redirect(redirect_uri))
                }
                Oauth2Redirect::Register => {
                    server.response_template(&client, TEMPLATE_AUTH_REGISTER_ACCEPT_OK)
                }
            }
        })
    })
}
