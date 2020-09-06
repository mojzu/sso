use crate::http_server::internal::*;

#[api_v2_operation(summary = "Create CSRF token")]
pub(crate) async fn post(
    server: Data<HttpServer>,
    req: HttpRequest,
    auth: BasicAuth,
) -> HttpResult<Json<ResponseCsrf>> {
    server_request!(&server, &req, async {
        let client = server.client_required(auth).await?;

        let res = server.postgres.csrf_insert(&client).await;

        server.response_json(res)
    })
}
