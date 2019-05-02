//! # User
use crate::server::{body_json_config, Data};
use actix_web::{middleware::identity::Identity, web, Error, HttpResponse};
use futures::{future, Future};

/// API version 1 service scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/user").service(
        web::resource("")
            .route(web::get().to_async(api_v1_list))
            // .route(web::post().data(body_json_config()).to_async(v1_create)),
    )
    // .service(
    //     web::resource("/{user_id}")
    //         .route(web::get().to_async(v1_read))
    //         .route(web::patch().data(body_json_config()).to_async(v1_update))
    //         .route(web::delete().to_async(v1_delete)),
    // )
}

/// API version 1 user list route.
pub fn api_v1_list(
    data: web::Data<Data>,
    id: Identity,
    query: web::Query<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {

}
