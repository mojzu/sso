use crate::{
    server::{
        route::{request_audit_meta, route_response_json},
        Data,
    },
    server_api::{
        path, AuditCreateBody, AuditCreateResponse, AuditListQuery, AuditListResponse,
        AuditReadResponse,
    },
    Audit, AuditList, AuditMeta, Key, ServerError, ServerResult, ServerValidateFromStr,
    ServerValidateFromValue,
};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
use serde_json::Value;
use uuid::Uuid;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::AUDIT)
        .service(
            web::resource(path::NONE)
                .route(web::get().to_async(list_handler))
                .route(web::post().to_async(create_handler)),
        )
        .service(web::resource(path::ID).route(web::get().to_async(read_handler)))
}

fn list_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let query = AuditListQuery::from_str(req.query_string());

    audit_meta
        .join(query)
        .and_then(|(audit_meta, query)| {
            web::block(move || list_inner(data.get_ref(), audit_meta, id, query))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn list_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    query: AuditListQuery,
) -> ServerResult<AuditListResponse> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            let list: AuditList = query.into();
            let data = Audit::list(data.driver(), service.as_ref(), &mut audit, &list)?;
            Ok(AuditListResponse {
                meta: list.into(),
                data,
            })
        })
        .map_err(Into::into)
}

fn create_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuditCreateBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || create_inner(data.get_ref(), audit_meta, id, &body))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn create_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &AuditCreateBody,
) -> ServerResult<AuditCreateResponse> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(move |(_, mut audit)| {
            audit
                .set_user_id(body.user_id.to_owned())
                .set_user_key_id(body.user_key_id.to_owned())
                .create(data.driver(), &body.type_, &body.data)
        })
        .map_err(Into::into)
        .map(|audit| AuditCreateResponse { data: audit })
}

fn read_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(Uuid,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);

    audit_meta
        .and_then(|audit_meta| {
            web::block(move || read_inner(data.get_ref(), audit_meta, id, path.0))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn read_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    audit_id: Uuid,
) -> ServerResult<AuditReadResponse> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Audit::read(data.driver(), service.as_ref(), &mut audit, audit_id)
        })
        .map_err(Into::into)
        .and_then(|audit| audit.ok_or_else(|| ServerError::NotFound))
        .map(|audit| AuditReadResponse { data: audit })
}
