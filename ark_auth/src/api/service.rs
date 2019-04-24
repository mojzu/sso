use crate::api::key::Key;
use crate::api::{authenticate, body_json_config, ApiData};
use crate::models::AuthService;
use actix_web::http::StatusCode;
use actix_web::middleware::identity::Identity;
use actix_web::{web, Error, HttpResponse};
use chrono::{DateTime, Utc};
use futures::Future;

/// Version 1 service routes scope.
pub fn v1_service() -> actix_web::Scope {
    web::scope("/service")
        .service(
            web::resource("")
                .route(web::get().to_async(v1_list))
                .route(web::post().data(body_json_config()).to_async(v1_create)),
        )
        .service(
            web::resource("/{service_id}")
                .route(web::get().to_async(v1_read))
                .route(web::patch().data(body_json_config()).to_async(v1_update))
                .route(web::delete().to_async(v1_delete)),
        )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    pub created_at: DateTime<Utc>,
    pub id: i64,
    pub name: String,
}

impl From<AuthService> for Service {
    fn from(service: AuthService) -> Self {
        Service {
            created_at: service.created_at,
            id: service.service_id,
            name: service.service_name,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub order: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse {
    pub data: Vec<Service>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBody {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateResponse {
    pub service: Service,
    pub key: Key,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBody {
    pub name: Option<String>,
}

pub fn v1_list(
    data: web::Data<ApiData>,
    id: Identity,
    query: web::Query<ListQuery>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let query = query.into_inner();

    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .service_list(
                    query.offset,
                    query.limit,
                    query.order.as_ref().map(|x| &**x),
                    service.service_id,
                )
                .map_err(Into::into)
                .map(|services| {
                    let data: Vec<Service> = services.into_iter().map(Into::into).collect();
                    ListResponse { data }
                })
        })
    })
    .map_err(Into::into)
    .map(|list_response| HttpResponse::build(StatusCode::OK).json(list_response))
}

pub fn v1_create(
    data: web::Data<ApiData>,
    id: Identity,
    body: web::Json<CreateBody>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let body = body.into_inner();

    web::block(move || {
        authenticate(&data, id).and_then(|_service| {
            data.db
                .service_create(&body.name, &body.url)
                .map_err(Into::into)
                .and_then(|service| {
                    data.db
                        .key_create(&body.name, service.service_id, None)
                        .map_err(Into::into)
                        .map(|key| CreateResponse {
                            service: service.into(),
                            key: key.into(),
                        })
                })
        })
    })
    .map_err(Into::into)
    .map(|create_response| HttpResponse::build(StatusCode::OK).json(create_response))
}

pub fn v1_read(
    data: web::Data<ApiData>,
    id: Identity,
    path: web::Path<(i64,)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .service_read_by_id(path.0, service.service_id)
                .map_err(Into::into)
                .map(Into::into)
        })
    })
    .map_err(Into::into)
    .map(|service: Service| HttpResponse::build(StatusCode::OK).json(service))
}

pub fn v1_update(
    data: web::Data<ApiData>,
    id: Identity,
    path: web::Path<(i64,)>,
    body: web::Json<UpdateBody>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let body = body.into_inner();

    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .service_update_by_id(path.0, service.service_id, body.name.as_ref().map(|x| &**x))
                .map_err(Into::into)
                .map(Into::into)
        })
    })
    .map_err(Into::into)
    .map(|service: Service| HttpResponse::build(StatusCode::OK).json(service))
}

pub fn v1_delete(
    data: web::Data<ApiData>,
    id: Identity,
    path: web::Path<(i64,)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .service_delete_by_id(path.0, service.service_id)
                .map_err(Into::into)
        })
    })
    .map_err(Into::into)
    .map(|_| HttpResponse::new(StatusCode::OK))
}
