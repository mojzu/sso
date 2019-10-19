use crate::{
    api::{self, ApiError, ValidateRequestQuery},
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
    User,
};
use actix_identity::Identity;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use futures::Future;
use uuid::Uuid;

pub fn route_v1_scope() -> Scope {
    web::scope(api::path::USER)
        .service(
            web::resource(api::path::NONE)
                .route(web::get().to_async(list_handler))
                .route(web::post().to_async(create_handler)),
        )
        .service(
            web::resource(api::path::ID)
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
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = api::UserListRequest::from_str_fut(req.query_string());

    audit_meta
        .join(request)
        .and_then(move |(audit_meta, request)| {
            web::block(move || api::user_list(data.driver(), audit_meta, id, request))
                .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_json)
}

fn create_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::UserCreateRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();
    let password_meta = User::password_meta(
        data.options().password_pwned_enabled(),
        data.client(),
        request.password.as_ref().map(|x| &**x),
    )
    .map_err(ApiError::BadRequest);

    audit_meta
        .join(password_meta)
        .and_then(move |(audit_meta, password_meta)| {
            web::block(move || {
                api::user_create(data.driver(), audit_meta, id, password_meta, request)
            })
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
