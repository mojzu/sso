use crate::http_server::internal::*;

#[api_v2_operation(summary = "OpenID connect discovery endpoint")]
pub async fn get(
    server: Data<HttpServer>,
    req: HttpRequest,
) -> HttpResult<Json<ResponseOpenidConfiguration>> {
    server_request!(&server, &req, async {
        let res = server.well_known_openid_configuration();

        server.response_json(res)
    })
}
