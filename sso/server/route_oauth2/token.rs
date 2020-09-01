use crate::server::internal::*;

#[api_v2_operation(summary = "OAuth2 token endpoint")]
pub async fn post_form(
    server: Data<Server>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Form<RequestOauth2Token>,
) -> HttpResult<HttpResponse> {
    post(server, req, auth, Json(body.into_inner())).await
}

#[api_v2_operation(summary = "OAuth2 token endpoint")]
pub async fn post_json(
    server: Data<Server>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestOauth2Token>,
) -> HttpResult<HttpResponse> {
    post(server, req, auth, body).await
}

async fn post(
    server: Data<Server>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestOauth2Token>,
) -> HttpResult<HttpResponse> {
    server_request!(&server, &req, async {
        let body = server_validate!(&server, body);

        let request = server
            .oauth2_token_parse_request(
                Some(&body.grant_type),
                body.code.as_deref(),
                body.redirect_uri.as_deref(),
                Some(&auth.id()),
                body.refresh_token.as_deref(),
                auth.secret().as_deref(),
            )
            .map_err(HttpError::bad_request)?;

        let client = server
            .client_from_secret(request.client_secret())
            .await
            .map_err(HttpError::unauthorized)?;

        let response = match request {
            oauth2::TokenParseRequest::Access(request) => {
                server.oauth2_access_token(&client, request).await
            }
            oauth2::TokenParseRequest::Refresh(request) => {
                server.oauth2_refresh_token(&client, request).await
            }
        }
        .map_err(HttpError::bad_request)?;

        server.response_json_untyped(response.serialize_json())
    })
}
