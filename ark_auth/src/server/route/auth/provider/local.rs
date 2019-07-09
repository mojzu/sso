use crate::core;
use crate::core::AuditMeta;
use crate::server::api::{
    AuthLoginBody, AuthLoginResponse, AuthPasswordMetaResponse, AuthResetPasswordBody,
    AuthResetPasswordConfirmBody, AuthUpdateEmailBody, AuthUpdateEmailRevokeBody,
    AuthUpdatePasswordBody, AuthUpdatePasswordRevokeBody,
};
use crate::server::route::auth::password_meta;
use crate::server::route::{request_audit_meta, route_response_empty, route_response_json};
use crate::server::{smtp, Data, Error, FromJsonValue};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::{future, Future};
use serde_json::Value;

// TODO(feature): Reset/update routes should not reveal if user exists, constant time?

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope("/local")
        .service(web::resource("/login").route(web::post().to_async(login_handler)))
        .service(
            web::scope("/reset")
                .service(
                    web::resource("/password").route(web::post().to_async(reset_password_handler)),
                )
                .service(
                    web::resource("/password/confirm")
                        .route(web::post().to_async(reset_password_confirm_handler)),
                ),
        )
        .service(
            web::scope("/update")
                .service(web::resource("/email").route(web::post().to_async(update_email_handler)))
                .service(
                    web::resource("/email/revoke")
                        .route(web::post().to_async(update_email_revoke_handler)),
                )
                .service(
                    web::resource("/password").route(web::post().to_async(update_password_handler)),
                )
                .service(
                    web::resource("/password/revoke")
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
            web::block(move || {
                let user_token = login_inner(data.get_ref(), audit_meta, id, &body)?;
                Ok((data, body, user_token))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, body, user_token)| {
            let password_meta = password_meta(data.get_ref(), Some(&body.password));
            let user_token = future::ok(user_token);
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
    body: &AuthLoginBody,
) -> Result<core::UserToken, Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::auth::login(
                data.driver(),
                &service,
                &mut audit,
                &body.email,
                &body.password,
                data.configuration().core_access_token_expires(),
                data.configuration().core_refresh_token_expires(),
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
            web::block(move || reset_password_inner(data.get_ref(), audit_meta, id, &body))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn reset_password_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &AuthResetPasswordBody,
) -> Result<(), Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            let (user, token) = core::auth::reset_password(
                data.driver(),
                &service,
                &mut audit,
                &body.email,
                data.configuration().core_access_token_expires(),
            )?;
            Ok((service, user, token))
        })
        .map_err(Into::into)
        .and_then(|(service, user, token)| {
            smtp::send_reset_password(data.configuration().smtp(), &service, &user, &token).or_else(
                |err| {
                    warn!("{}", err);
                    Ok(())
                },
            )
        })
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
            web::block(move || {
                let password_confirm =
                    reset_password_confirm_inner(data.get_ref(), audit_meta, id, &body)?;
                Ok((data, body, password_confirm))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, body, _password_confirm)| {
            password_meta(data.get_ref(), Some(&body.password))
        })
        .map(|meta| AuthPasswordMetaResponse { meta })
        .then(route_response_json)
}

fn reset_password_confirm_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &AuthResetPasswordConfirmBody,
) -> Result<usize, Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::auth::reset_password_confirm(
                data.driver(),
                &service,
                &mut audit,
                &body.token,
                &body.password,
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
            web::block(move || update_email_inner(data.get_ref(), audit_meta, id, &body))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_email_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &AuthUpdateEmailBody,
) -> Result<(), Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            let (user, old_email, token) = core::auth::update_email(
                data.driver(),
                &service,
                &mut audit,
                body.key.as_ref().map(|x| &**x),
                body.token.as_ref().map(|x| &**x),
                &body.password,
                &body.new_email,
                data.configuration().core_revoke_token_expires(),
            )?;
            Ok((service, user, old_email, token))
        })
        .map_err(Into::into)
        .and_then(|(service, user, old_email, token)| {
            smtp::send_update_email(
                data.configuration().smtp(),
                &service,
                &user,
                &old_email,
                &token,
            )
            .or_else(|err| {
                warn!("{}", err);
                Ok(())
            })
        })
}

fn update_email_revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthUpdateEmailRevokeBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || update_email_revoke_inner(data.get_ref(), audit_meta, id, &body))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_email_revoke_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &AuthUpdateEmailRevokeBody,
) -> Result<usize, Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::auth::update_email_revoke(data.driver(), &service, &mut audit, &body.token)
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
            web::block(move || {
                update_password_inner(data.get_ref(), audit_meta, id, &body)?;
                Ok((data, body))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, body)| password_meta(data.get_ref(), Some(&body.password)))
        .map(|meta| AuthPasswordMetaResponse { meta })
        .then(route_response_json)
}

fn update_password_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &AuthUpdatePasswordBody,
) -> Result<(), Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            let (user, token) = core::auth::update_password(
                data.driver(),
                &service,
                &mut audit,
                body.key.as_ref().map(|x| &**x),
                body.token.as_ref().map(|x| &**x),
                &body.password,
                &body.new_password,
                data.configuration().core_revoke_token_expires(),
            )?;
            Ok((service, user, token))
        })
        .map_err(Into::into)
        .and_then(|(service, user, token)| {
            smtp::send_update_password(data.configuration().smtp(), &service, &user, &token)
                .or_else(|err| {
                    warn!("{}", err);
                    Ok(())
                })
        })
}

fn update_password_revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthUpdatePasswordRevokeBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || update_password_revoke_inner(data.get_ref(), audit_meta, id, &body))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_password_revoke_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &AuthUpdatePasswordRevokeBody,
) -> Result<usize, Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::auth::update_password_revoke(data.driver(), &service, &mut audit, &body.token)
        })
        .map_err(Into::into)
}
