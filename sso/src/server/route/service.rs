use crate::{
    api::{self, ValidateRequestQuery},
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
};
use actix_identity::Identity;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use futures::Future;
use uuid::Uuid;

fn list_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = api::ServiceListRequest::from_str_fut(req.query_string());

    audit_meta
        .join(request)
        .and_then(move |(audit_meta, request)| {
            web::block(move || api::service_list(data.driver(), audit_meta, id, request))
                .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_json)
}

fn create_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::ServiceCreateRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || api::service_create(data.driver(), audit_meta, id, request))
                .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_json)
}

fn read_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(Uuid,)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let (service_id,) = path.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || api::service_read(data.driver(), audit_meta, id, service_id))
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
    body: web::Json<api::ServiceUpdateRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let (service_id,) = path.into_inner();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                api::service_update(data.driver(), audit_meta, id, service_id, request)
            })
            .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_json)
}

fn delete_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(Uuid,)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let (service_id,) = path.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || api::service_delete(data.driver(), audit_meta, id, service_id))
                .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_empty)
}
