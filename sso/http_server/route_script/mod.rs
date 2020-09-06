use crate::http_server::internal::*;

#[api_v2_operation()]
pub(crate) async fn zxcvbn_get(
    server: Data<HttpServer>,
    req: HttpRequest,
) -> HttpResult<HttpResponse> {
    server_request!(&server, &req, async {
        Ok(HttpResponse::Ok()
            .content_type("text/javascript; charset=utf-8")
            .body(include_str!("zxcvbn.js")))
    })
}

#[api_v2_operation()]
pub(crate) async fn password_strength_get(
    server: Data<HttpServer>,
    req: HttpRequest,
) -> HttpResult<HttpResponse> {
    server_request!(&server, &req, async {
        Ok(HttpResponse::Ok()
            .content_type("text/javascript; charset=utf-8")
            .body(include_str!("password_strength.js")))
    })
}
