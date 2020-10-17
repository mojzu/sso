use crate::http_server::internal::*;

#[api_v2_operation(summary = "Change password endpoint")]
pub async fn get(server: Data<HttpServer>, req: HttpRequest) -> HttpResult<HttpResponse> {
    server_request!(&server, &req, async {
        let mut audit = Audit::from_http_request("sso_well_known_change_password", &req);
        let id = server
            .request_identity_required(&mut audit, &req)
            .await
            .map_err(Error::Oauth2)
            .map_err(HttpError::Unauthorized)?;
        let client = server
            .client_from_user_id(&mut audit, id)
            .await
            .map_err(HttpError::Forbidden)?;

        let redirect_uri = server.uri_auth_password_update(&client, None);
        Ok(server.response_redirect(redirect_uri))
    })
}
