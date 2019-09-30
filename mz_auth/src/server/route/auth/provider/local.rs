use crate::{
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
    server_api::{
        path, AuthLoginBody, AuthLoginResponse, AuthPasswordMetaResponse, AuthResetPasswordBody,
        AuthResetPasswordConfirmBody, AuthTokenBody, AuthUpdateEmailBody, AuthUpdatePasswordBody,
    },
    AuditData, AuditMeta, Auth, Key, ServerResult, ServerValidateFromValue, User, UserToken,
};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
use serde_json::Value;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::LOCAL)
        .service(web::resource(path::LOGIN).route(web::post().to_async(login_handler)))
        .service(
            web::scope(path::RESET_PASSWORD)
                .service(
                    web::resource(path::NONE).route(web::post().to_async(reset_password_handler)),
                )
                .service(
                    web::resource(path::CONFIRM)
                        .route(web::post().to_async(reset_password_confirm_handler)),
                ),
        )
        .service(
            web::scope(path::UPDATE_EMAIL)
                .service(
                    web::resource(path::NONE).route(web::post().to_async(update_email_handler)),
                )
                .service(
                    web::resource(path::REVOKE)
                        .route(web::post().to_async(update_email_revoke_handler)),
                ),
        )
        .service(
            web::scope(path::UPDATE_PASSWORD)
                .service(
                    web::resource(path::NONE).route(web::post().to_async(update_password_handler)),
                )
                .service(
                    web::resource(path::REVOKE)
                        .route(web::post().to_async(update_password_revoke_handler)),
                ),
        )
}

fn login_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthLoginBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            let password_meta = User::password_meta(
                data.options().password_pwned_enabled(),
                data.client(),
                Some(&body.password),
            )
            .map_err(Into::into);
            let user_token = web::block(move || login_inner(data.get_ref(), audit_meta, id, body))
                .map_err(Into::into);
            password_meta.join(user_token)
        })
        .map(|(meta, user_token)| AuthLoginResponse {
            meta,
            data: user_token,
        })
        .then(route_response_json)
}

fn login_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: AuthLoginBody,
) -> ServerResult<UserToken> {
    Key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Auth::login(
                data.driver(),
                &service,
                &mut audit,
                body.email,
                body.password,
                data.options().access_token_expires(),
                data.options().refresh_token_expires(),
            )
        })
        .map_err(Into::into)
}

fn reset_password_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthResetPasswordBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || reset_password_inner(data.get_ref(), audit_meta, id, body))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn reset_password_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: AuthResetPasswordBody,
) -> ServerResult<()> {
    Key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Auth::reset_password(
                data.driver(),
                data.notify(),
                &service,
                &mut audit,
                body.email,
                data.options().access_token_expires(),
            )
        })
        .map_err(Into::into)
}

fn reset_password_confirm_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthResetPasswordConfirmBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            let password_meta = User::password_meta(
                data.options().password_pwned_enabled(),
                data.client(),
                Some(&body.password),
            )
            .map_err(Into::into);
            let password_confirm = web::block(move || {
                reset_password_confirm_inner(data.get_ref(), audit_meta, id, body)
            })
            .map_err(Into::into);
            password_meta.join(password_confirm)
        })
        .map(|(meta, _)| AuthPasswordMetaResponse { meta })
        .then(route_response_json)
}

fn reset_password_confirm_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: AuthResetPasswordConfirmBody,
) -> ServerResult<()> {
    Key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Auth::reset_password_confirm(
                data.driver(),
                &service,
                &mut audit,
                body.token,
                body.password,
            )
        })
        .map_err(Into::into)
}

fn update_email_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthUpdateEmailBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || update_email_inner(data.get_ref(), audit_meta, id, body))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_email_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: AuthUpdateEmailBody,
) -> ServerResult<()> {
    Key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Auth::update_email(
                data.driver(),
                data.notify(),
                &service,
                &mut audit,
                body.key,
                body.token,
                body.password,
                body.new_email,
                data.options().revoke_token_expires(),
            )
        })
        .map_err(Into::into)
}

fn update_email_revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthTokenBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || {
                update_email_revoke_inner(
                    data.get_ref(),
                    audit_meta,
                    id,
                    body.token,
                    body.audit.map(Into::into),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_email_revoke_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    token: String,
    audit_data: Option<AuditData>,
) -> ServerResult<usize> {
    Key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Auth::update_email_revoke(
                data.driver(),
                &service,
                &mut audit,
                token,
                audit_data.as_ref(),
            )
        })
        .map_err(Into::into)
}

fn update_password_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthUpdatePasswordBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            let password_meta = User::password_meta(
                data.options().password_pwned_enabled(),
                data.client(),
                Some(&body.password),
            )
            .map_err(Into::into);
            let update_password =
                web::block(move || update_password_inner(data.get_ref(), audit_meta, id, body))
                    .map_err(Into::into);
            password_meta.join(update_password)
        })
        .map(|(meta, _update_password)| AuthPasswordMetaResponse { meta })
        .then(route_response_json)
}

fn update_password_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: AuthUpdatePasswordBody,
) -> ServerResult<()> {
    Key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Auth::update_password(
                data.driver(),
                data.notify(),
                &service,
                &mut audit,
                body.key,
                body.token,
                body.password,
                body.new_password,
                data.options().revoke_token_expires(),
            )
        })
        .map_err(Into::into)
}

fn update_password_revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthTokenBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || {
                update_password_revoke_inner(
                    data.get_ref(),
                    audit_meta,
                    id,
                    body.token,
                    body.audit.map(Into::into),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_password_revoke_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    token: String,
    audit_data: Option<AuditData>,
) -> ServerResult<usize> {
    Key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Auth::update_password_revoke(
                data.driver(),
                &service,
                &mut audit,
                token,
                audit_data.as_ref(),
            )
        })
        .map_err(Into::into)
}
