use crate::server::internal::*;

#[api_v2_operation(summary = "Delete user")]
pub async fn post(
    server: Data<Server>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestUserDelete>,
) -> HttpResult<Json<()>> {
    server_request!(&server, &req, async {
        let body = server_validate!(&server, body);

        let _client = server.client_required(auth).await?;

        let res = server.postgres.user_delete(body.id).await;

        server.response_json(res)
    })
}
