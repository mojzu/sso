use crate::core;
use crate::server::route::auth::{password_meta, PasswordMeta};
use crate::server::route::{route_json_config, route_response_empty, route_response_json};
use crate::server::{smtp, validate, Data, Error, FromJsonValue};
use actix_identity::Identity;
use actix_web::{web, HttpResponse};
use futures::{future, Future};
use serde_json::Value;
use validator::Validate;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope("/local")
        .service(
            web::resource("/login")
                .data(route_json_config())
                .route(web::post().to_async(login_handler)),
        )
        .service(
            web::scope("/reset")
                .service(
                    web::resource("/password")
                        .data(route_json_config())
                        .route(web::post().to_async(reset_password_handler)),
                )
                .service(
                    web::resource("/password/confirm")
                        .data(route_json_config())
                        .route(web::post().to_async(reset_password_confirm_handler)),
                ),
        )
        .service(
            web::scope("/update")
                .service(
                    web::resource("/email")
                        .data(route_json_config())
                        .route(web::post().to_async(update_email_handler)),
                )
                .service(
                    web::resource("/email/revoke")
                        .data(route_json_config())
                        .route(web::post().to_async(update_email_revoke_handler)),
                )
                .service(
                    web::resource("/password")
                        .data(route_json_config())
                        .route(web::post().to_async(update_password_handler)),
                )
                .service(
                    web::resource("/password/revoke")
                        .data(route_json_config())
                        .route(web::post().to_async(update_password_revoke_handler)),
                ),
        )
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct LoginBody {
    #[validate(email)]
    pub email: String,
    #[validate(custom = "validate::password")]
    pub password: String,
}

impl FromJsonValue<LoginBody> for LoginBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub meta: PasswordMeta,
    pub data: core::UserToken,
}

fn login_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    LoginBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || {
                let user_token = login_inner(data.get_ref(), id, &body)?;
                Ok((data, body, user_token))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, body, user_token)| {
            let password_meta = password_meta(data.get_ref(), Some(&body.password));
            let user_token = future::ok(user_token);
            password_meta.join(user_token)
        })
        .map(|(meta, user_token)| LoginResponse {
            meta,
            data: user_token,
        })
        .then(route_response_json)
}

fn login_inner(
    data: &Data,
    id: Option<String>,
    body: &LoginBody,
) -> Result<core::UserToken, Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| {
            core::auth::login(
                data.driver(),
                &service,
                &body.email,
                &body.password,
                data.configuration().core_access_token_expires(),
                data.configuration().core_refresh_token_expires(),
            )
        })
        .map_err(Into::into)
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ResetPasswordTemplateBody {
    #[validate(custom = "validate::email_subject")]
    pub subject: String,
    #[validate(custom = "validate::email_text")]
    pub text: String,
    #[validate(custom = "validate::email_link_text")]
    pub link_text: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ResetPasswordBody {
    #[validate(email)]
    pub email: String,
    pub template: Option<ResetPasswordTemplateBody>,
}

impl FromJsonValue<ResetPasswordBody> for ResetPasswordBody {}

fn reset_password_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    ResetPasswordBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || reset_password_inner(data.get_ref(), id, &body)).map_err(Into::into)
        })
        .then(route_response_empty)
}

fn reset_password_inner(
    data: &Data,
    id: Option<String>,
    body: &ResetPasswordBody,
) -> Result<(), Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| {
            let (user, token) = core::auth::reset_password(
                data.driver(),
                &service,
                &body.email,
                data.configuration().core_access_token_expires(),
            )?;
            Ok((service, body, user, token))
        })
        .map_err(Into::into)
        .and_then(|(service, body, user, token)| {
            smtp::send_reset_password(
                data.configuration().smtp(),
                &service,
                &user,
                &token,
                body.template.as_ref(),
            )
            .or_else(|err| {
                warn!("{}", err);
                Ok(())
            })
        })
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ResetPasswordConfirmBody {
    #[validate(custom = "validate::token")]
    pub token: String,
    #[validate(custom = "validate::password")]
    pub password: String,
}

impl FromJsonValue<ResetPasswordConfirmBody> for ResetPasswordConfirmBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordConfirmResponse {
    pub meta: PasswordMeta,
}

fn reset_password_confirm_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    ResetPasswordConfirmBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || {
                let password_confirm = reset_password_confirm_inner(data.get_ref(), id, &body)?;
                Ok((data, body, password_confirm))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, body, _password_confirm)| {
            password_meta(data.get_ref(), Some(&body.password))
        })
        .map(|meta| ResetPasswordConfirmResponse { meta })
        .then(route_response_json)
}

fn reset_password_confirm_inner(
    data: &Data,
    id: Option<String>,
    body: &ResetPasswordConfirmBody,
) -> Result<usize, Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| {
            core::auth::reset_password_confirm(data.driver(), &service, &body.token, &body.password)
        })
        .map_err(Into::into)
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateEmailTemplateBody {
    #[validate(custom = "validate::email_subject")]
    pub subject: String,
    #[validate(custom = "validate::email_text")]
    pub text: String,
    #[validate(custom = "validate::email_link_text")]
    pub link_text: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateEmailBody {
    #[validate(custom = "validate::key")]
    pub key: Option<String>,
    #[validate(custom = "validate::token")]
    pub token: Option<String>,
    #[validate(custom = "validate::password")]
    pub password: String,
    #[validate(email)]
    pub new_email: String,
    pub template: Option<UpdateEmailTemplateBody>,
}

impl FromJsonValue<UpdateEmailBody> for UpdateEmailBody {}

fn update_email_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    UpdateEmailBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || update_email_inner(data.get_ref(), id, &body)).map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_email_inner(
    data: &Data,
    id: Option<String>,
    body: &UpdateEmailBody,
) -> Result<(), Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| {
            let (user, old_email, token) = core::auth::update_email(
                data.driver(),
                &service,
                body.key.as_ref().map(|x| &**x),
                body.token.as_ref().map(|x| &**x),
                &body.password,
                &body.new_email,
                data.configuration().core_revoke_token_expires(),
            )?;
            Ok((service, body, user, old_email, token))
        })
        .map_err(Into::into)
        .and_then(|(service, body, user, old_email, token)| {
            smtp::send_update_email(
                data.configuration().smtp(),
                &service,
                &user,
                &old_email,
                &token,
                body.template.as_ref(),
            )
            .or_else(|err| {
                warn!("{}", err);
                Ok(())
            })
        })
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateEmailRevokeBody {
    #[validate(custom = "validate::token")]
    pub token: String,
}

impl FromJsonValue<UpdateEmailRevokeBody> for UpdateEmailRevokeBody {}

fn update_email_revoke_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    UpdateEmailRevokeBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || update_email_revoke_inner(data.get_ref(), id, &body))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_email_revoke_inner(
    data: &Data,
    id: Option<String>,
    body: &UpdateEmailRevokeBody,
) -> Result<usize, Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| core::auth::update_email_revoke(data.driver(), &service, &body.token))
        .map_err(Into::into)
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdatePasswordTemplateBody {
    #[validate(custom = "validate::email_subject")]
    pub subject: String,
    #[validate(custom = "validate::email_text")]
    pub text: String,
    #[validate(custom = "validate::email_link_text")]
    pub link_text: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdatePasswordBody {
    #[validate(custom = "validate::key")]
    pub key: Option<String>,
    #[validate(custom = "validate::token")]
    pub token: Option<String>,
    #[validate(custom = "validate::password")]
    pub password: String,
    #[validate(custom = "validate::password")]
    pub new_password: String,
    pub template: Option<UpdatePasswordTemplateBody>,
}

impl FromJsonValue<UpdatePasswordBody> for UpdatePasswordBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePasswordResponse {
    pub meta: PasswordMeta,
}

fn update_password_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    UpdatePasswordBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || {
                let update_password = update_password_inner(data.get_ref(), id, &body)?;
                Ok((data, body, update_password))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, body, _update_password)| {
            password_meta(data.get_ref(), Some(&body.password))
        })
        .map(|meta| UpdatePasswordResponse { meta })
        .then(route_response_json)
}

fn update_password_inner(
    data: &Data,
    id: Option<String>,
    body: &UpdatePasswordBody,
) -> Result<(), Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| {
            let (user, token) = core::auth::update_password(
                data.driver(),
                &service,
                body.key.as_ref().map(|x| &**x),
                body.token.as_ref().map(|x| &**x),
                &body.password,
                &body.new_password,
                data.configuration().core_revoke_token_expires(),
            )?;
            Ok((service, body, user, token))
        })
        .map_err(Into::into)
        .and_then(|(service, body, user, token)| {
            smtp::send_update_password(
                data.configuration().smtp(),
                &service,
                &user,
                &token,
                body.template.as_ref(),
            )
            .or_else(|err| {
                warn!("{}", err);
                Ok(())
            })
        })
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdatePasswordRevokeBody {
    #[validate(custom = "validate::token")]
    pub token: String,
}

impl FromJsonValue<UpdatePasswordRevokeBody> for UpdatePasswordRevokeBody {}

fn update_password_revoke_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    UpdatePasswordRevokeBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || update_password_revoke_inner(data.get_ref(), id, &body))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn update_password_revoke_inner(
    data: &Data,
    id: Option<String>,
    body: &UpdatePasswordRevokeBody,
) -> Result<usize, Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| {
            core::auth::update_password_revoke(data.driver(), &service, &body.token)
        })
        .map_err(Into::into)
}
