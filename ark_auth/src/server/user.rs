//! # User
use crate::{
    core, server,
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
            .route(web::get().to_async(v1_list))
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
#[serde(deny_unknown_fields)]
pub struct ListQuery {
    #[validate(custom = "validate_unsigned")]
    pub gt: Option<i64>,
    #[validate(custom = "validate_unsigned")]
    pub lt: Option<i64>,
    #[validate(custom = "validate_unsigned")]
    pub limit: Option<i64>,
}

impl ValidateFromValue<ListQuery> for ListQuery {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMetaResponse {
    pub gt: Option<i64>,
    pub lt: Option<i64>,
    pub limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    pub meta: ListMetaResponse,
    pub data: Vec<core::User>,
}

/// API version 1 user list route.
pub fn v1_list(
    data: web::Data<Data>,
    id: Identity,
    query: web::Query<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    ListQuery::from_value(query.into_inner())
        .and_then(|query| list_inner(data, id, query))
        .then(|result| route_response(result))
}

/// User list handler.
fn list_inner(
    data: web::Data<Data>,
    id: Option<String>,
    query: ListQuery,
) -> impl Future<Item = ListResponse, Error = server::Error> {
    web::block(move || {
        core::service_authenticate(data.driver(), id)
            .and_then(|service| {
                let limit = query.limit.unwrap_or(10);
                let (gt, lt, users) = match query.lt {
                    Some(lt) => {
                        let users =
                            core::user_list_where_id_lt(data.driver(), &service, lt, limit)?;
                        (None, Some(lt), users)
                    }
                    None => {
                        let gt = query.gt.unwrap_or(0);
                        let users =
                            core::user_list_where_id_gt(data.driver(), &service, gt, limit)?;
                        (Some(gt), None, users)
                    }
                };

                Ok(ListResponse {
                    meta: ListMetaResponse { gt, lt, limit },
                    data: users,
                })
            })
            .map_err(Into::into)
    })
    .map_err(Into::into)
}
