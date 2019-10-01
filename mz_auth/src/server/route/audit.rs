use crate::{
    api_types::{AuditCreateRequest, AuditListRequest},
    server::{
        route::{request_audit_meta, route_response_json},
        Data,
    },
    server_api::path,
    Api, ApiValidateRequestQuery, ServerError,
};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
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
    let request = AuditListRequest::from_str_fut(req.query_string()).map_err(ServerError::Core);

    audit_meta
        .join(request)
        .and_then(move |(audit_meta, request)| {
            web::block(move || {
                Api::audit_list(data.driver(), id, audit_meta, request).map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn create_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuditCreateRequest>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::audit_create(data.driver(), id, audit_meta, request).map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn read_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(Uuid,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let audit_id = path.0;

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::audit_read(data.driver(), id, audit_meta, audit_id).map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}
