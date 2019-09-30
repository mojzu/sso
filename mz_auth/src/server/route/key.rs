use crate::{
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
    server_api::{
        path, KeyCreateBody, KeyListQuery, KeyListResponse, KeyReadResponse, KeyUpdateBody,
    },
    AuditMeta, CoreError, Key, KeyList, ServerError, ServerResult, ServerValidateFromStr,
    ServerValidateFromValue,
};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
use serde_json::Value;
use uuid::Uuid;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::KEY)
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
    let query = KeyListQuery::from_str(req.query_string());

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
    query: KeyListQuery,
) -> ServerResult<KeyListResponse> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            let list: KeyList = query.into();
            let data = Key::list(data.driver(), service.as_ref(), &mut audit, &list)?;
            Ok(KeyListResponse {
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
    let body = KeyCreateBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || create_inner(data.get_ref(), audit_meta, id, body))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn create_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: KeyCreateBody,
) -> ServerResult<KeyReadResponse> {
    // If service ID is some, root key is required to create service keys.
    match body.service_id {
        Some(service_id) => {
            Key::authenticate_root(data.driver(), audit_meta, id).and_then(|mut audit| {
                match body.user_id {
                    // User ID is defined, creating user key for service.
                    Some(user_id) => Key::create_user(
                        data.driver(),
                        &mut audit,
                        body.is_enabled,
                        body.allow_key,
                        body.allow_token,
                        body.allow_totp,
                        body.name,
                        service_id,
                        user_id,
                    ),
                    // Creating service key.
                    None => Key::create_service(
                        data.driver(),
                        &mut audit,
                        body.is_enabled,
                        body.name,
                        service_id,
                    ),
                }
            })
        }
        None => Key::authenticate_service(data.driver(), audit_meta, id).and_then(
            |(service, mut audit)| {
                match body.user_id {
                    // User ID is defined, creating user key for service.
                    Some(user_id) => Key::create_user(
                        data.driver(),
                        &mut audit,
                        body.is_enabled,
                        body.allow_key,
                        body.allow_token,
                        body.allow_totp,
                        body.name,
                        service.id,
                        user_id,
                    ),
                    // Service cannot create service keys.
                    None => Err(CoreError::BadRequest),
                }
            },
        ),
    }
    .map_err(Into::into)
    .map(|key| KeyReadResponse { data: key })
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
    key_id: Uuid,
) -> ServerResult<KeyReadResponse> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Key::read_opt(data.driver(), service.as_ref(), &mut audit, key_id)
        })
        .map_err(Into::into)
        .and_then(|key| key.ok_or_else(|| ServerError::NotFound))
        .map(|key| KeyReadResponse { data: key })
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
    let body = KeyUpdateBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || update_inner(data.get_ref(), audit_meta, id, path.0, body))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn update_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    key_id: Uuid,
    body: KeyUpdateBody,
) -> ServerResult<KeyReadResponse> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Key::update(
                data.driver(),
                service.as_ref(),
                &mut audit,
                key_id,
                body.is_enabled,
                body.allow_key,
                body.allow_token,
                body.allow_totp,
                None,
                body.name,
            )
        })
        .map_err(Into::into)
        .map(|key| KeyReadResponse { data: key })
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
    key_id: Uuid,
) -> ServerResult<usize> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Key::delete(data.driver(), service.as_ref(), &mut audit, key_id)
        })
        .map_err(Into::into)
}
