use crate::http_server::internal::*;

#[api_v2_operation(summary = "Read user access")]
pub(crate) async fn post(
    server: Data<HttpServer>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestAccessRead>,
) -> HttpResult<Json<ResponseAccessMany>> {
    server_request!(&server, &req, async {
        let body = server_validate!(&server, body);

        let client = server.client_required(auth).await?;

        let res = server.postgres.access_read_many(&client, body).await;

        server.response_json(res)
    })
}
