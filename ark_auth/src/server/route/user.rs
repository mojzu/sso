use crate::core;
use crate::core::{AuditMeta, UserQuery};
use crate::server::api::{
    path, UserCreateBody, UserCreateResponse, UserListQuery, UserListResponse, UserReadResponse,
    UserUpdateBody,
};
use crate::server::route::auth::password_meta;
use crate::server::route::{request_audit_meta, route_response_empty, route_response_json};
use crate::server::{Data, Error, FromJsonValue};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::{future, Future};
use serde_json::Value;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::USER)
        .service(
            web::resource(path::NONE)
                .route(web::get().to_async(list_handler))
                .route(web::post().to_async(create_handler)),
        )
        .service(
            web::resource(path::ID)
                .route(web::get().to_async(read_handler))
                .route(web::patch().to_async(update_handler))
                .route(web::delete().to_async(delete_handler)),
        )
}

fn list_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    query: web::Query<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let query = UserListQuery::from_value(query.into_inner());

    audit_meta
        .join(query)
        .and_then(|(audit_meta, query)| {
            web::block(move || list_inner(data.get_ref(), audit_meta, id, query.into()))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn list_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    query: UserQuery,
) -> Result<UserListResponse, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            let user_ids = core::user::list(data.driver(), service.as_ref(), &mut audit, &query)?;
            Ok(UserListResponse {
                meta: query,
                data: user_ids,
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
    let body = UserCreateBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || {
                let user = create_inner(data.get_ref(), audit_meta, id, &body)?;
                Ok((data, body, user))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, body, user)| {
            let password_meta = password_meta(data.get_ref(), body.password.as_ref().map(|x| &**x));
            let user = future::ok(user);
            password_meta.join(user)
        })
        .map(|(meta, user)| UserCreateResponse { meta, data: user })
        .then(route_response_json)
}

fn create_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &UserCreateBody,
) -> Result<core::User, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::user::create(
                data.driver(),
                service.as_ref(),
                &mut audit,
                body.is_enabled,
                &body.name,
                &body.email,
                body.password.as_ref().map(|x| &**x),
            )
        })
        .map_err(Into::into)
}

fn read_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(String,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);

    audit_meta
        .and_then(|audit_meta| {
            web::block(move || read_inner(data.get_ref(), audit_meta, id, &path.0))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn read_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    user_id: &str,
) -> Result<UserReadResponse, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::user::read_by_id(data.driver(), service.as_ref(), &mut audit, user_id)
        })
        .map_err(Into::into)
        .and_then(|user| user.ok_or_else(|| Error::NotFound))
        .map(|user| UserReadResponse { data: user })
}

fn update_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(String,)>,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = UserUpdateBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || update_inner(data.get_ref(), audit_meta, id, &path.0, &body))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn update_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    user_id: &str,
    body: &UserUpdateBody,
) -> Result<UserReadResponse, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::user::update_by_id(
                data.driver(),
                service.as_ref(),
                &mut audit,
                user_id,
                body.is_enabled,
                body.name.as_ref().map(|x| &**x),
            )
        })
        .map_err(Into::into)
        .map(|user| UserReadResponse { data: user })
}

fn delete_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(String,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);

    audit_meta
        .and_then(|audit_meta| {
            web::block(move || delete_inner(data.get_ref(), audit_meta, id, &path.0))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn delete_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    user_id: &str,
) -> Result<usize, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::user::delete_by_id(data.driver(), service.as_ref(), &mut audit, user_id)
        })
        .map_err(Into::into)
}
