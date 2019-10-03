use crate::{
    api_path,
    api_type::{
        AuthLoginRequest, AuthResetPasswordConfirmRequest, AuthResetPasswordRequest,
        AuthTokenRequest, AuthUpdateEmailRequest, AuthUpdatePasswordRequest,
    },
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
    Api, User,
};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(api_path::LOCAL)
        .service(web::resource(api_path::LOGIN).route(web::post().to_async(login_handler)))
        .service(
            web::scope(api_path::RESET_PASSWORD)
                .service(
                    web::resource(api_path::NONE)
                        .route(web::post().to_async(reset_password_handler)),
                )
                .service(
                    web::resource(api_path::CONFIRM)
                        .route(web::post().to_async(reset_password_confirm_handler)),
                ),
        )
        .service(
            web::scope(api_path::UPDATE_EMAIL)
                .service(
                    web::resource(api_path::NONE).route(web::post().to_async(update_email_handler)),
                )
                .service(
                    web::resource(api_path::REVOKE)
                        .route(web::post().to_async(update_email_revoke_handler)),
                ),
        )
        .service(
            web::scope(api_path::UPDATE_PASSWORD)
                .service(
                    web::resource(api_path::NONE)
                        .route(web::post().to_async(update_password_handler)),
                )
                .service(
                    web::resource(api_path::REVOKE)
                        .route(web::post().to_async(update_password_revoke_handler)),
                ),
        )
}

fn login_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuthLoginRequest>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();
    let password_meta = User::password_meta(
        data.options().password_pwned_enabled(),
        data.client(),
        Some(&request.password),
    )
    .map_err(Into::into);

    audit_meta
        .join(password_meta)
        .and_then(move |(audit_meta, password_meta)| {
            web::block(move || {
                Api::auth_provider_local_login(
                    data.driver(),
                    id,
                    audit_meta,
                    password_meta,
                    request,
                    data.options().access_token_expires(),
                    data.options().refresh_token_expires(),
                )
                .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn reset_password_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuthResetPasswordRequest>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::auth_provider_local_reset_password(
                    data.driver(),
                    id,
                    audit_meta,
                    request,
                    data.notify(),
                    data.options().refresh_token_expires(),
                )
                .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn reset_password_confirm_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuthResetPasswordConfirmRequest>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();
    let password_meta = User::password_meta(
        data.options().password_pwned_enabled(),
        data.client(),
        Some(&request.password),
    )
    .map_err(Into::into);

    audit_meta
        .join(password_meta)
        .and_then(move |(audit_meta, password_meta)| {
            web::block(move || {
                Api::auth_provider_local_reset_password_confirm(
                    data.driver(),
                    id,
                    audit_meta,
                    password_meta,
                    request,
                )
                .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn update_email_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuthUpdateEmailRequest>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::auth_provider_local_update_email(
                    data.driver(),
                    id,
                    audit_meta,
                    request,
                    data.notify(),
                    data.options().revoke_token_expires(),
                )
                .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_email_revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuthTokenRequest>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::auth_provider_local_update_email_revoke(data.driver(), id, audit_meta, request)
                    .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_password_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuthUpdatePasswordRequest>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();
    let password_meta = User::password_meta(
        data.options().password_pwned_enabled(),
        data.client(),
        Some(&request.password),
    )
    .map_err(Into::into);

    audit_meta
        .join(password_meta)
        .and_then(move |(audit_meta, password_meta)| {
            web::block(move || {
                Api::auth_provider_local_update_password(
                    data.driver(),
                    id,
                    audit_meta,
                    password_meta,
                    request,
                    data.notify(),
                    data.options().revoke_token_expires(),
                )
                .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn update_password_revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuthTokenRequest>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::auth_provider_local_update_password_revoke(
                    data.driver(),
                    id,
                    audit_meta,
                    request,
                )
                .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}
