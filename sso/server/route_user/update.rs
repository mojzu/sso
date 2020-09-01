use crate::server::internal::*;

#[api_v2_operation(summary = "Update user")]
pub async fn post(
    server: Data<Server>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestUserUpdate>,
) -> HttpResult<Json<ResponseUser>> {
    server_request!(&server, &req, async {
        let body = server_validate!(&server, body);

        let client = server.client_required(auth).await?;

        let res = server.postgres.user_update(&client, body).await;

        server.response_json(res)
    })
}
