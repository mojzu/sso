use crate::server::internal::*;

#[api_v2_operation(summary = "Verify user API key")]
pub async fn post(
    server: Data<Server>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestApiKeyVerify>,
) -> HttpResult<Json<ResponseApiKey>> {
    server_request!(&server, &req, async {
        let body = server_validate!(&server, body);

        let client = server.client_required(auth).await?;

        let access = server.postgres.api_key_verify(&client, body).await;

        server.response_json(access)
    })
}
