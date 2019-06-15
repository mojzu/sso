use crate::core;
use crate::server::route::auth::{password_meta, PasswordMeta};
use crate::server::route::{route_json_config, route_response_empty, route_response_json};
use crate::server::{smtp, validate, Data, Error, FromJsonValue};
use actix_web::{middleware::identity::Identity, web, HttpResponse};
use futures::{future, Future};
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
    body: web::Json<serde_json::Value>,
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
                data.configuration().token_expiration_time(),
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
    body: web::Json<serde_json::Value>,
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
                data.configuration().token_expiration_time(),
            )?;
            Ok((service, body, user, token))
        })
        .map_err(Into::into)
        .and_then(|(service, body, user, token)| {
            smtp::send_reset_password(
                data.configuration().smtp(),
                &service,
                &user,
                &token.token,
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
    body: web::Json<serde_json::Value>,
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
