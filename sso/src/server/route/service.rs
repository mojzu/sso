use crate::{
    api_path,
    api_type::{ServiceCreateRequest, ServiceListRequest, ServiceUpdateRequest},
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
    Api, ApiValidateRequestQuery, ServerError,
};
use actix_identity::Identity;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use futures::Future;
use uuid::Uuid;

pub fn route_v1_scope() -> Scope {
    web::scope(api_path::SERVICE)
        .service(
            web::resource(api_path::NONE)
                .route(web::get().to_async(list_handler))
                .route(web::post().to_async(create_handler)),
        )
        .service(
            web::resource(api_path::ID)
                .route(web::get().to_async(read_handler))
                .route(web::patch().to_async(update_handler))
                .route(web::delete().to_async(delete_handler)),
        )
}

fn list_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = ServiceListRequest::from_str_fut(req.query_string()).map_err(ServerError::Core);

    audit_meta
        .join(request)
        .and_then(move |(audit_meta, request)| {
            web::block(move || {
                Api::service_list(data.driver(), id, audit_meta, request).map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn create_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<ServiceCreateRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::service_create(data.driver(), id, audit_meta, request).map_err(Into::into)
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
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let service_id = path.0;

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::service_read(data.driver(), id, audit_meta, service_id).map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn update_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(Uuid,)>,
    body: web::Json<ServiceUpdateRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let service_id = path.0;
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::service_update(data.driver(), id, audit_meta, service_id, request)
                    .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn delete_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(Uuid,)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let service_id = path.0;

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::service_delete(data.driver(), id, audit_meta, service_id).map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}
