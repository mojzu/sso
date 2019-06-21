use crate::core;
use crate::server::route::auth::{password_meta, PasswordMeta};
use crate::server::route::{route_json_config, route_response_empty, route_response_json};
use crate::server::{validate, Data, Error, FromJsonValue};
use actix_identity::Identity;
use actix_web::{web, HttpResponse};
use futures::{future, Future};
use serde_json::Value;
use validator::Validate;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope("/user")
        .service(
            web::resource("")
                .data(route_json_config())
                .route(web::get().to_async(list_handler))
                .route(web::post().to_async(create_handler)),
        )
        .service(
            web::resource("/{user_id}")
                .data(route_json_config())
                .route(web::get().to_async(read_handler))
                .route(web::patch().to_async(update_handler))
                .route(web::delete().to_async(delete_handler)),
        )
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ListQuery {
    #[validate(custom = "validate::id")]
    pub gt: Option<String>,
    #[validate(custom = "validate::id")]
    pub lt: Option<String>,
    #[validate(custom = "validate::limit")]
    pub limit: Option<i64>,
    #[validate(email)]
    pub email_eq: Option<String>,
}

impl FromJsonValue<ListQuery> for ListQuery {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListMetaResponse {
    pub gt: Option<String>,
    pub lt: Option<String>,
    pub limit: i64,
    pub email_eq: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse {
    pub meta: ListMetaResponse,
    pub data: Vec<String>,
}

fn list_handler(
    data: web::Data<Data>,
    id: Identity,
    query: web::Query<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    ListQuery::from_value(query.into_inner())
        .and_then(|query| {
            web::block(move || list_inner(data.get_ref(), id, query)).map_err(Into::into)
        })
        .then(route_response_json)
}

fn list_inner(data: &Data, id: Option<String>, query: ListQuery) -> Result<ListResponse, Error> {
    core::key::authenticate(data.driver(), id)
        .and_then(|service| {
            if let Some(email_eq) = &query.email_eq {
                let users =
                    core::user::list_where_email_eq(data.driver(), service.as_ref(), &email_eq, 1)?;
                return Ok(ListResponse {
                    meta: ListMetaResponse {
                        gt: None,
                        lt: None,
                        limit: 1,
                        email_eq: Some(email_eq.to_owned()),
                    },
                    data: users,
                });
            }

            // TODO(refactor): Configurable default/max limits.
            let limit = query.limit.unwrap_or(10);
            let (gt, lt, users) = match query.lt {
                Some(lt) => {
                    let users =
                        core::user::list_where_id_lt(data.driver(), service.as_ref(), &lt, limit)?;
                    (None, Some(lt), users)
                }
                None => {
                    let gt = query.gt.unwrap_or_else(|| "".to_owned());
                    let users =
                        core::user::list_where_id_gt(data.driver(), service.as_ref(), &gt, limit)?;
                    (Some(gt), None, users)
                }
            };
            Ok(ListResponse {
                meta: ListMetaResponse {
                    gt,
                    lt,
                    limit,
                    email_eq: None,
                },
                data: users,
            })
        })
        .map_err(Into::into)
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateBody {
    pub is_active: bool,
    #[validate(custom = "validate::name")]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom = "validate::password")]
    pub password: Option<String>,
}

impl FromJsonValue<CreateBody> for CreateBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateResponse {
    pub meta: PasswordMeta,
    pub data: core::User,
}

fn create_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    CreateBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || {
                let user = create_inner(data.get_ref(), id, &body)?;
                Ok((data, body, user))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, body, user)| {
            let password_meta = password_meta(data.get_ref(), body.password.as_ref().map(|x| &**x));
            let user = future::ok(user);
            password_meta.join(user)
        })
        .map(|(meta, user)| CreateResponse { meta, data: user })
        .then(route_response_json)
}

fn create_inner(data: &Data, id: Option<String>, body: &CreateBody) -> Result<core::User, Error> {
    core::key::authenticate(data.driver(), id)
        .and_then(|service| {
            core::user::create(
                data.driver(),
                service.as_ref(),
                body.is_active,
                &body.name,
                &body.email,
                body.password.as_ref().map(|x| &**x),
            )
        })
        .map_err(Into::into)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadResponse {
    pub data: core::User,
}

fn read_handler(
    data: web::Data<Data>,
    id: Identity,
    path: web::Path<(String,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    web::block(move || read_inner(data.get_ref(), id, &path.0))
        .map_err(Into::into)
        .then(route_response_json)
}

fn read_inner(data: &Data, id: Option<String>, user_id: &str) -> Result<ReadResponse, Error> {
    core::key::authenticate(data.driver(), id)
        .and_then(|service| core::user::read_by_id(data.driver(), service.as_ref(), user_id))
        .map_err(Into::into)
        .and_then(|user| user.ok_or_else(|| Error::NotFound))
        .map(|user| ReadResponse { data: user })
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateBody {
    pub is_active: Option<bool>,
    #[validate(custom = "validate::name")]
    pub name: Option<String>,
}

impl FromJsonValue<UpdateBody> for UpdateBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResponse {
    pub data: core::User,
}

fn update_handler(
    data: web::Data<Data>,
    id: Identity,
    path: web::Path<(String,)>,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    UpdateBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || update_inner(data.get_ref(), id, &path.0, &body)).map_err(Into::into)
        })
        .then(route_response_json)
}

fn update_inner(
    data: &Data,
    id: Option<String>,
    user_id: &str,
    body: &UpdateBody,
) -> Result<UpdateResponse, Error> {
    core::key::authenticate(data.driver(), id)
        .and_then(|service| {
            core::user::update_by_id(
                data.driver(),
                service.as_ref(),
                user_id,
                body.is_active,
                body.name.as_ref().map(|x| &**x),
            )
        })
        .map_err(Into::into)
        .map(|user| UpdateResponse { data: user })
}

fn delete_handler(
    data: web::Data<Data>,
    id: Identity,
    path: web::Path<(String,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    web::block(move || delete_inner(data.get_ref(), id, &path.0))
        .map_err(Into::into)
        .then(route_response_empty)
}

fn delete_inner(data: &Data, id: Option<String>, user_id: &str) -> Result<usize, Error> {
    core::key::authenticate(data.driver(), id)
        .and_then(|service| core::user::delete_by_id(data.driver(), service.as_ref(), user_id))
        .map_err(Into::into)
}
