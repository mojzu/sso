//! # User
use crate::{
    server,
    server::{route_json_config, route_response, validate_unsigned, Data, ValidateFromValue},
};
use actix_web::{middleware::identity::Identity, web, Error, HttpResponse};
use chrono::{DateTime, Utc};
use futures::{future, Future};
use validator::Validate;

/// API version 1 service scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/user").service(
        web::resource("")
            .route(web::get().to_async(api_v1_list))
            // .route(web::post().data(route_json_config()).to_async(v1_create)),
    )
    .service(
        web::resource("/{user_id}")
            // .route(web::get().to_async(v1_read))
            // .route(web::patch().data(route_json_config()).to_async(v1_update))
            // .route(web::delete().to_async(v1_delete)),
    )
}

/// List query.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ListQuery {
    #[validate(custom = "validate_unsigned")]
    pub gt: Option<i64>,
    #[validate(custom = "validate_unsigned")]
    pub lt: Option<i64>,
    #[validate(custom = "validate_unsigned")]
    pub limit: Option<i64>,
}

impl ValidateFromValue<ListQuery> for ListQuery {}

// TODO(refactor): Refactor User into driver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    pub data: Vec<User>,
}

/// API version 1 user list route.
pub fn api_v1_list(
    data: web::Data<Data>,
    id: Identity,
    query: web::Query<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    ListQuery::from_value(query.into_inner())
        .and_then(|query| {
            web::block(move || list_inner(data.get_ref(), id, query)).map_err(Into::into)
        })
        .then(|result| route_response(result))
}

/// User list handler.
fn list_inner(
    data: &Data,
    id: Option<String>,
    query: ListQuery,
) -> Result<ListResponse, server::Error> {
    unimplemented!();
}
