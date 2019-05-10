use crate::server::{
    auth::{password_meta, PasswordMeta},
    route_json_config, route_response_empty, route_response_json, validate_email_link_text,
    validate_email_subject, validate_email_text, validate_password, validate_token, Data, Error,
    ValidateFromValue,
};
use crate::{core, smtp};
use actix_web::{middleware::identity::Identity, web, HttpResponse};
use futures::Future;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct PasswordTemplateBody {
    #[validate(custom = "validate_email_subject")]
    pub subject: String,
    #[validate(custom = "validate_email_text")]
    pub text: String,
    #[validate(custom = "validate_email_link_text")]
    pub link_text: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
struct PasswordBody {
    #[validate(email)]
    email: String,
    template: Option<PasswordTemplateBody>,
}

impl ValidateFromValue<PasswordBody> for PasswordBody {}

fn password_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    PasswordBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || password_inner(data.get_ref(), id, &body)).map_err(Into::into)
        })
        .then(route_response_empty)
}

fn password_inner(data: &Data, id: Option<String>, body: &PasswordBody) -> Result<(), Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| {
            let (user, token) = core::auth::reset_password(data.driver(), &service, &body.email)?;
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
        })
        .or_else(|err| {
            warn!("{}", err);
            Ok(())
        })
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
struct PasswordConfirmBody {
    #[validate(custom = "validate_token")]
    token: String,
    #[validate(custom = "validate_password")]
    password: String,
}

impl ValidateFromValue<PasswordConfirmBody> for PasswordConfirmBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordConfirmResponse {
    pub meta: PasswordMeta,
}

fn password_confirm_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    PasswordConfirmBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || {
                let password_confirm = password_confirm_inner(data.get_ref(), id, &body)?;
                Ok((data, body, password_confirm))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, body, _password_confirm)| {
            password_meta(data.get_ref(), Some(&body.password))
        })
        .map(|meta| PasswordConfirmResponse { meta })
        .then(route_response_json)
}

fn password_confirm_inner(
    data: &Data,
    id: Option<String>,
    body: &PasswordConfirmBody,
) -> Result<usize, Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| {
            core::auth::reset_password_confirm(data.driver(), &service, &body.token, &body.password)
        })
        .map_err(Into::into)
}

/// Version 1 API authentication reset scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/reset")
        .service(
            web::resource("/password")
                .data(route_json_config())
                .route(web::post().to_async(password_handler)),
        )
        .service(
            web::resource("/password/confirm")
                .data(route_json_config())
                .route(web::post().to_async(password_confirm_handler)),
        )
}
