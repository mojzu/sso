use crate::api::{
    auth::{check_password_meta, validate_name, validate_password, PasswordMetaResponse},
    authenticate, body_json_config, ApiData, ApiError, BodyFromValue,
};
use crate::models::AuthUser;
use actix_web::{
    http::StatusCode, middleware::identity::Identity, web, Error, HttpResponse, ResponseError,
};
use chrono::{DateTime, Utc};
use futures::{future, Future};
use validator::Validate;

/// Version 1 user routes scope.
pub fn v1_service() -> actix_web::Scope {
    web::scope("/user")
        .service(
            web::resource("")
                .route(web::get().to_async(v1_list))
                .route(web::post().data(body_json_config()).to_async(v1_create)),
        )
        .service(
            web::resource("/{user_id}")
                .route(web::get().to_async(v1_read))
                .route(web::patch().data(body_json_config()).to_async(v1_update))
                .route(web::delete().to_async(v1_delete)),
        )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: i64,
    pub name: String,
    pub email: String,
}

impl From<AuthUser> for User {
    fn from(user: AuthUser) -> Self {
        User {
            created_at: user.created_at,
            updated_at: user.updated_at,
            id: user.user_id,
            name: user.user_name,
            email: user.user_email,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub order: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    pub data: Vec<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateBody {
    #[validate(custom = "validate_name")]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom = "validate_password")]
    pub password: Option<String>,
}

impl BodyFromValue<CreateBody> for CreateBody {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResponse {
    meta: PasswordMetaResponse,
    data: User,
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

    list_inner(data, id, query).then(|r| match r {
        Ok(r) => future::ok(HttpResponse::Ok().json(r)),
        Err(e) => future::ok(e.error_response()),
    })
}

fn list_inner(
    data: web::Data<ApiData>,
    id: Option<String>,
    query: ListQuery,
) -> impl Future<Item = ListResponse, Error = ApiError> {
    web::block(move || {
        authenticate(&data, id).and_then(|_service| {
            data.db
                .user_list(
                    query.offset,
                    query.limit,
                    query.order.as_ref().map(|x| &**x),
                )
                .map_err(Into::into)
                .map(|users| {
                    let data: Vec<User> = users.into_iter().map(Into::into).collect();
                    ListResponse { data }
                })
        })
    })
    .map_err(Into::into)
}

pub fn v1_create(
    data: web::Data<ApiData>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    CreateBody::from_value(body.into_inner())
        .and_then(move |body| v1_create_inner(data, id, body))
        .then(|user| match user {
            Ok(user) => future::ok(HttpResponse::Ok().json(user)),
            Err(e) => future::ok(e.error_response()),
        })
}

fn v1_create_inner(
    data: web::Data<ApiData>,
    id: Option<String>,
    body: CreateBody,
) -> impl Future<Item = CreateResponse, Error = ApiError> {
    let (data1, body1) = (data.clone(), body.clone());

    web::block(move || {
        authenticate(&data, id).and_then(|_service| {
            data.db
                .user_create(
                    &body.name,
                    &body.email,
                    body.password.as_ref().map(|x| &**x),
                )
                .map_err(Into::into)
        })
    })
    .map_err(Into::into)
    .and_then(move |user| {
        let password_meta = match body1.password {
            Some(password) => future::Either::A(check_password_meta(&data1, &password)),
            None => future::Either::B(future::ok(PasswordMetaResponse::default())),
        };
        let user = future::ok(user);
        password_meta.join(user)
    })
    .map(|(meta, user)| CreateResponse {
        meta,
        data: user.into(),
    })
}

pub fn v1_read(
    data: web::Data<ApiData>,
    id: Identity,
    path: web::Path<(i64,)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    web::block(move || {
        authenticate(&data, id).and_then(|_service| {
            data.db
                .user_read_by_id(path.0)
                .map_err(Into::into)
                .map(Into::into)
        })
    })
    .map_err(Into::into)
    .map(|user: User| HttpResponse::build(StatusCode::OK).json(user))
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
        authenticate(&data, id).and_then(|_service| {
            data.db
                .user_update_by_id(path.0, body.name.as_ref().map(|x| &**x))
                .map_err(Into::into)
                .map(Into::into)
        })
    })
    .map_err(Into::into)
    .map(|user: User| HttpResponse::build(StatusCode::OK).json(user))
}

pub fn v1_delete(
    data: web::Data<ApiData>,
    id: Identity,
    path: web::Path<(i64,)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    web::block(move || {
        authenticate(&data, id)
            .and_then(|_service| data.db.user_delete_by_id(path.0).map_err(Into::into))
    })
    .map_err(Into::into)
    .map(|_| HttpResponse::new(StatusCode::OK))
}
