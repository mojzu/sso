use crate::http_server::internal::*;

#[api_v2_operation(summary = "Read user API key")]
pub(crate) async fn post(
    server: Data<HttpServer>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestApiKeyRead>,
) -> HttpResult<Json<ResponseApiKeyMany>> {
    server_request!(&server, &req, async {
        let body = server_validate!(&server, body);

        let client = server.client_required(auth).await?;

        let res = server.postgres.api_key_read(&client, body).await;

        server.response_json(res)
    })
}
