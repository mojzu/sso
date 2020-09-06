use crate::http_server::internal::*;

#[api_v2_operation(summary = "Create audit log")]
pub(crate) async fn post(
    server: Data<HttpServer>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestAuditCreate>,
) -> HttpResult<Json<ResponseAudit>> {
    server_request!(&server, &req, async {
        let body = server_validate!(&server, body);

        let client = server.client_required(auth).await?;

        let audit = Audit::from_create_request(&client, body);

        let res = server.postgres.audit_insert(audit).await;

        server.response_json(res)
    })
}
