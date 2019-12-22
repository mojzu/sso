use crate::{
    api::{self, ApiError},
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
};
use actix_identity::Identity;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use futures::Future;

fn login_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthLoginRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();
    let password_meta = api::password_meta(
        data.client(),
        data.options().password_pwned_enabled(),
        Some(request.password.clone()),
    )
    .map_err(ApiError::BadRequest);

    audit_meta
        .join(password_meta)
        .and_then(move |(audit_meta, password_meta)| {
            web::block(move || {
                api::auth_provider_local_login(
                    data.driver(),
                    audit_meta,
                    id,
                    password_meta,
                    request,
                    data.options().access_token_expires(),
                    data.options().refresh_token_expires(),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn register_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthRegisterRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                api::auth_provider_local_register(
                    data.driver(),
                    audit_meta,
                    id,
                    request,
                    data.options().access_token_expires(),
                    data.smtp_email(),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn register_confirm_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthRegisterConfirmRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();
    let password_meta = api::password_meta(
        data.client(),
        data.options().password_pwned_enabled(),
        request.password.clone(),
    )
    .map_err(ApiError::BadRequest);

    audit_meta
        .join(password_meta)
        .and_then(move |(audit_meta, password_meta)| {
            web::block(move || {
                api::auth_provider_local_register_confirm(
                    data.driver(),
                    audit_meta,
                    id,
                    password_meta,
                    request,
                    data.options().revoke_token_expires(),
                    data.smtp_email(),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn register_revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthTokenRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                api::auth_provider_local_register_revoke(data.driver(), audit_meta, id, request)
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn reset_password_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthResetPasswordRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                api::auth_provider_local_reset_password(
                    data.driver(),
                    audit_meta,
                    id,
                    request,
                    data.options().access_token_expires(),
                    data.smtp_email(),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn reset_password_confirm_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthResetPasswordConfirmRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();
    let password_meta = api::password_meta(
        data.client(),
        data.options().password_pwned_enabled(),
        Some(request.password.clone()),
    )
    .map_err(ApiError::BadRequest);

    audit_meta
        .join(password_meta)
        .and_then(move |(audit_meta, password_meta)| {
            web::block(move || {
                api::auth_provider_local_reset_password_confirm(
                    data.driver(),
                    audit_meta,
                    id,
                    password_meta,
                    request,
                    data.options().revoke_token_expires(),
                    data.smtp_email(),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn reset_password_revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthTokenRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                api::auth_provider_local_reset_password_revoke(
                    data.driver(),
                    audit_meta,
                    id,
                    request,
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_email_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthUpdateEmailRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                api::auth_provider_local_update_email(
                    data.driver(),
                    audit_meta,
                    id,
                    request,
                    data.options().revoke_token_expires(),
                    data.smtp_email(),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_email_revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthTokenRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                api::auth_provider_local_update_email_revoke(data.driver(), audit_meta, id, request)
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_password_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthUpdatePasswordRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();
    let password_meta = api::password_meta(
        data.client(),
        data.options().password_pwned_enabled(),
        Some(request.password.clone()),
    )
    .map_err(ApiError::BadRequest);

    audit_meta
        .join(password_meta)
        .and_then(move |(audit_meta, password_meta)| {
            web::block(move || {
                api::auth_provider_local_update_password(
                    data.driver(),
                    audit_meta,
                    id,
                    password_meta,
                    request,
                    data.options().revoke_token_expires(),
                    data.smtp_email(),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn update_password_revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthTokenRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                api::auth_provider_local_update_password_revoke(
                    data.driver(),
                    audit_meta,
                    id,
                    request,
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}
