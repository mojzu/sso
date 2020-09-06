use crate::http_server::internal::*;

#[api_v2_operation(summary = "User logout interface")]
pub async fn get(
    server: Data<HttpServer>,
    req: HttpRequest,
    query: Query<RequestAuthQuery>,
) -> oauth2::Result<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_auth_logout", &req);
        let query = server_oauth2_validate!(&server, query);
        let query = AuthClientId::parse(query)?;

        let client = server.client_from_id(&mut audit, &query).await?;

        let ident = server.request_identity(&req).await;
        ident.forget();

        server.response_template(&client, TEMPLATE_AUTH_LOGOUT)
    })
}
