mod key;
mod provider;
mod token;

use crate::server_api::path;
use actix_web::web;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::AUTH)
        .service(provider::route_v1_scope())
        .service(key::route_v1_scope())
        .service(token::route_v1_scope())
}
