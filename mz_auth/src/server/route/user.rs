use crate::{
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
    server_api::{
        path, UserCreateBody, UserCreateResponse, UserListQuery, UserListResponse,
        UserReadResponse, UserUpdateBody,
    },
    AuditMeta, Key, ServerError, ServerResult, ServerValidateFromStr, ServerValidateFromValue,
    User,
};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::{future, Future};
use serde_json::Value;
use uuid::Uuid;

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
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let query = UserListQuery::from_str(req.query_string());

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
    query: UserListQuery,
) -> ServerResult<UserListResponse> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            let list = query.to_user_list();
            let data = User::list(data.driver(), service.as_ref(), &mut audit, &list)?;
            Ok(UserListResponse { data })
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
            let password_meta = User::password_meta(
                data.options().password_pwned_enabled(),
                data.client(),
                body.password.as_ref().map(|x| &**x),
            )
            .map_err(Into::into);
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
) -> ServerResult<User> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            User::create(
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
    user_id: Uuid,
) -> ServerResult<UserReadResponse> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            User::read_by_id(data.driver(), service.as_ref(), &mut audit, user_id)
        })
        .map_err(Into::into)
        .and_then(|user| user.ok_or_else(|| ServerError::NotFound))
        .map(|user| UserReadResponse { data: user })
}

fn update_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(Uuid,)>,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = UserUpdateBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || update_inner(data.get_ref(), audit_meta, id, path.0, &body))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn update_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    user_id: Uuid,
    body: &UserUpdateBody,
) -> ServerResult<UserReadResponse> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            User::update_by_id(
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
    path: web::Path<(Uuid,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);

    audit_meta
        .and_then(|audit_meta| {
            web::block(move || delete_inner(data.get_ref(), audit_meta, id, path.0))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn delete_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    user_id: Uuid,
) -> ServerResult<usize> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            User::delete_by_id(data.driver(), service.as_ref(), &mut audit, user_id)
        })
        .map_err(Into::into)
}
