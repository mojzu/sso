use crate::http_server::internal::*;

#[api_v2_operation(summary = "Read authenticated client")]
pub(crate) async fn post(
    server: Data<HttpServer>,
    req: HttpRequest,
    auth: BasicAuth,
) -> HttpResult<Json<ResponseClient>> {
    server_request!(&server, &req, async {
        let client = server.client_required(auth).await?;

        let res: ResponseClient = (&client).into();

        server.response_json(Ok(res))
    })
}
