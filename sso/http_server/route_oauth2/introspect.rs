use crate::http_server::internal::*;

#[api_v2_operation(summary = "OAuth2 token introspection endpoint")]
pub(crate) async fn post_form(
    server: Data<HttpServer>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Form<RequestOauth2Introspect>,
) -> HttpResult<HttpResponse> {
    post(server, req, auth, Json(body.into_inner())).await
}

#[api_v2_operation(summary = "OAuth2 token introspection endpoint")]
pub(crate) async fn post_json(
    server: Data<HttpServer>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestOauth2Introspect>,
) -> HttpResult<HttpResponse> {
    post(server, req, auth, body).await
}

async fn post(
    server: Data<HttpServer>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestOauth2Introspect>,
) -> HttpResult<HttpResponse> {
    server_request!(&server, &req, async {
        let body = server_validate!(&server, body);

        let request = server
            .oauth2_introspect_parse_request(Some(&body.token), auth.secret().as_deref())
            .map_err(HttpError::bad_request)?;

        let client = server
            .client_from_secret(&auth.id(), request.client_secret())
            .await
            .map_err(HttpError::unauthorized)?;

        let response = server
            .oauth2_introspection(&client, request)
            .await
            .map_err(HttpError::bad_request)?;

        server.response_json_untyped(response.serialize_json())
    })
}
