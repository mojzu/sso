use crate::core;
use crate::core::{AuditMeta, KeyQuery};
use crate::server::api::{
    KeyCreateBody, KeyListQuery, KeyListResponse, KeyReadResponse, KeyUpdateBody,
};
use crate::server::route::{request_audit_meta, route_response_empty, route_response_json};
use crate::server::{Data, Error, FromJsonValue};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
use serde_json::Value;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope("/key")
        .service(
            web::resource("")
                .route(web::get().to_async(list_handler))
                .route(web::post().to_async(create_handler)),
        )
        .service(
            web::resource("/{key_id}")
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
    let query = KeyListQuery::from_value(query.into_inner());

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
    query: KeyQuery,
) -> Result<KeyListResponse, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            let key_ids = core::key::list(data.driver(), service.as_ref(), &mut audit, &query)?;
            Ok(KeyListResponse {
                meta: query,
                data: key_ids,
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
            web::block(move || create_inner(data.get_ref(), audit_meta, id, &body))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn create_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &KeyCreateBody,
) -> Result<KeyReadResponse, Error> {
    // If service ID is some, root key is required to create service keys.
    match body.service_id.as_ref() {
        Some(service_id) => {
            core::key::authenticate_root(data.driver(), audit_meta, id).and_then(|mut audit| {
                match body.user_id.as_ref() {
                    // User ID is defined, creating user key for service.
                    Some(user_id) => core::key::create_user(
                        data.driver(),
                        &mut audit,
                        body.is_enabled,
                        &body.name,
                        &service_id,
                        &user_id,
                    ),
                    // Creating service key.
                    None => core::key::create_service(
                        data.driver(),
                        &mut audit,
                        body.is_enabled,
                        &body.name,
                        &service_id,
                    ),
                }
            })
        }
        None => core::key::authenticate_service(data.driver(), audit_meta, id).and_then(
            |(service, mut audit)| {
                match body.user_id.as_ref() {
                    // User ID is defined, creating user key for service.
                    Some(user_id) => core::key::create_user(
                        data.driver(),
                        &mut audit,
                        body.is_enabled,
                        &body.name,
                        &service.id,
                        &user_id,
                    ),
                    // Service cannot create service keys.
                    None => Err(core::Error::BadRequest),
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
    key_id: &str,
) -> Result<KeyReadResponse, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::key::read_by_id(data.driver(), service.as_ref(), &mut audit, key_id)
        })
        .map_err(Into::into)
        .and_then(|key| key.ok_or_else(|| Error::NotFound))
        .map(|key| KeyReadResponse { data: key })
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
    let body = KeyUpdateBody::from_value(body.into_inner());

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
    key_id: &str,
    body: &KeyUpdateBody,
) -> Result<KeyReadResponse, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::key::update_by_id(
                data.driver(),
                service.as_ref(),
                &mut audit,
                key_id,
                body.is_enabled,
                None,
                body.name.as_ref().map(|x| &**x),
            )
        })
        .map_err(Into::into)
        .map(|key| KeyReadResponse { data: key })
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
    key_id: &str,
) -> Result<usize, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::key::delete_by_id(data.driver(), service.as_ref(), &mut audit, key_id)
        })
        .map_err(Into::into)
}
