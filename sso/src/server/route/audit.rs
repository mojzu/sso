use crate::{
    api::{self, ValidateRequestQuery},
    server::{
        route::{request_audit_meta, route_response_json},
        Data,
    },
};
use actix_identity::Identity;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use futures::Future;
use uuid::Uuid;

pub fn route_v1_scope() -> Scope {
    web::scope(api::path::AUDIT)
        .service(
            web::resource(api::path::NONE)
                .route(web::get().to_async(list_handler))
                .route(web::post().to_async(create_handler)),
        )
        .service(
            web::resource(api::path::ID)
                .route(web::get().to_async(read_handler))
                .route(web::patch().to_async(update_handler)),
        )
}

fn read_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(Uuid,)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let (audit_id,) = path.into_inner();
    let request = api::AuditReadRequest::from_str_fut(req.query_string());

    audit_meta
        .join(request)
        .and_then(move |(audit_meta, request)| {
            web::block(move || api::audit_read(data.driver(), audit_meta, id, audit_id, request))
                .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_json)
}

fn update_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(Uuid,)>,
    body: web::Json<api::AuditUpdateRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let (audit_id,) = path.into_inner();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || api::audit_update(data.driver(), audit_meta, id, audit_id, request))
                .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_json)
}
