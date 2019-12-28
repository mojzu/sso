use crate::{
    api::{self, ApiError, ValidateRequestQuery},
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
};
use actix_identity::Identity;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use futures::Future;
use uuid::Uuid;

fn read_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(Uuid,)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let (user_id,) = path.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || api::user_read(data.driver(), audit_meta, id, user_id))
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
    body: web::Json<api::UserUpdateRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let (user_id,) = path.into_inner();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || api::user_update(data.driver(), audit_meta, id, user_id, request))
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
    let (user_id,) = path.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || api::user_delete(data.driver(), audit_meta, id, user_id))
                .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_empty)
}
