use crate::http_server::internal::*;

#[api_v2_operation(summary = "Create user API key")]
pub(crate) async fn post(
    server: Data<HttpServer>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestApiKeyCreate>,
) -> HttpResult<Json<ResponseApiKey>> {
    server_request!(&server, &req, async {
        let body = server_validate!(&server, body);

        let client = server.client_required(auth).await?;

        let res = server.postgres.api_key_create(&client, body).await;

        server.response_json(res)
    })
}
