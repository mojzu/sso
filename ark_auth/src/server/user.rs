use crate::{
    core,
    server::{
        auth::{password_meta, PasswordMeta},
        route_json_config, route_response_empty, route_response_json, validate_name,
        validate_password, validate_unsigned, Data, Error, ValidateFromValue,
    },
};
use actix_web::{middleware::identity::Identity, web, HttpResponse};
use futures::{future, Future};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
struct ListQuery {
    #[validate(custom = "validate_unsigned")]
    gt: Option<i64>,
    #[validate(custom = "validate_unsigned")]
    lt: Option<i64>,
    #[validate(custom = "validate_unsigned")]
    limit: Option<i64>,
}

impl ValidateFromValue<ListQuery> for ListQuery {}

#[derive(Debug, Serialize, Deserialize)]
struct ListMetaResponse {
    gt: Option<i64>,
    lt: Option<i64>,
    limit: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListResponse {
    meta: ListMetaResponse,
    data: Vec<core::User>,
}

fn list_handler(
    data: web::Data<Data>,
    id: Identity,
    query: web::Query<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    ListQuery::from_value(query.into_inner())
        .and_then(|query| {
            web::block(move || list_inner(data.get_ref(), id, &query)).map_err(Into::into)
        })
        .then(|res| route_response_json(res))
}

fn list_inner(data: &Data, id: Option<String>, query: &ListQuery) -> Result<ListResponse, Error> {
    core::service::authenticate(data.driver(), id)
        .and_then(|service| {
            let limit = query.limit.unwrap_or(10);
            let (gt, lt, users) = match query.lt {
                Some(lt) => {
                    let users = core::user::list_where_id_lt(data.driver(), &service, lt, limit)?;
                    (None, Some(lt), users)
                }
                None => {
                    let gt = query.gt.unwrap_or(0);
                    let users = core::user::list_where_id_gt(data.driver(), &service, gt, limit)?;
                    (Some(gt), None, users)
                }
            };

            Ok(ListResponse {
                meta: ListMetaResponse { gt, lt, limit },
                data: users,
            })
        })
        .map_err(Into::into)
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
struct CreateBody {
    #[validate(custom = "validate_name")]
    name: String,
    #[validate(email)]
    email: String,
    #[validate(custom = "validate_password")]
    password: Option<String>,
}

impl ValidateFromValue<CreateBody> for CreateBody {}

#[derive(Debug, Serialize, Deserialize)]
struct CreateResponse {
    meta: PasswordMeta,
    data: core::User,
}

fn create_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
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
        .then(|res| route_response_json(res))
}

fn create_inner(data: &Data, id: Option<String>, body: &CreateBody) -> Result<core::User, Error> {
    core::service::authenticate(data.driver(), id)
        .and_then(|service| {
            core::user::create(
                data.driver(),
                &service,
                &body.name,
                &body.email,
                body.password.as_ref().map(|x| &**x),
            )
        })
        .map_err(Into::into)
}

#[derive(Debug, Serialize, Deserialize)]
struct ReadResponse {
    data: core::User,
}

fn read_handler(
    data: web::Data<Data>,
    id: Identity,
    path: web::Path<(i64,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    web::block(move || read_inner(data.get_ref(), id, path.0))
        .map_err(Into::into)
        .then(|res| route_response_json(res))
}

fn read_inner(data: &Data, id: Option<String>, user_id: i64) -> Result<ReadResponse, Error> {
    core::service::authenticate(data.driver(), id)
        .and_then(|service| core::user::read_by_id(data.driver(), &service, user_id))
        .map_err(Into::into)
        .and_then(|user| user.ok_or_else(|| Error::NotFound))
        .map(|user| ReadResponse { data: user })
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
struct UpdateBody {
    #[validate(custom = "validate_name")]
    name: Option<String>,
}

impl ValidateFromValue<UpdateBody> for UpdateBody {}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateResponse {
    data: core::User,
}

fn update_handler(
    data: web::Data<Data>,
    id: Identity,
    path: web::Path<(i64,)>,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    UpdateBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || update_inner(data.get_ref(), id, path.0, &body)).map_err(Into::into)
        })
        .then(|res| route_response_json(res))
}

fn update_inner(
    data: &Data,
    id: Option<String>,
    user_id: i64,
    body: &UpdateBody,
) -> Result<UpdateResponse, Error> {
    core::service::authenticate(data.driver(), id)
        .and_then(|service| {
            core::user::update_by_id(
                data.driver(),
                &service,
                user_id,
                body.name.as_ref().map(|x| &**x),
            )
        })
        .map_err(Into::into)
        .map(|user| UpdateResponse { data: user })
}

fn delete_handler(
    data: web::Data<Data>,
    id: Identity,
    path: web::Path<(i64,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    web::block(move || delete_inner(data.get_ref(), id, path.0))
        .map_err(Into::into)
        .then(|res| route_response_empty(res))
}

fn delete_inner(data: &Data, id: Option<String>, user_id: i64) -> Result<usize, Error> {
    core::service::authenticate(data.driver(), id)
        .and_then(|service| core::user::delete_by_id(data.driver(), &service, user_id))
        .map_err(Into::into)
}

/// API version 1 user scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/user")
        .service(
            web::resource("")
                .route(web::get().to_async(list_handler))
                .route(
                    web::post()
                        .data(route_json_config())
                        .to_async(create_handler),
                ),
        )
        .service(
            web::resource("/{user_id}")
                .route(web::get().to_async(read_handler))
                .route(
                    web::patch()
                        .data(route_json_config())
                        .to_async(update_handler),
                )
                .route(web::delete().to_async(delete_handler)),
        )
}
