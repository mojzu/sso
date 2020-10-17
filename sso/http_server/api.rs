use crate::http_server::*;
use actix_web::{guard, HttpResponse};
use paperclip::actix::{
    api_v2_operation,
    web::{self, Data, HttpRequest, Json},
};

/// Server API
pub(crate) struct ServerApi;

impl ServerApi {
    fn public_v2_oauth2() -> web::Scope {
        web::scope("/oauth2")
            .route("/authorize", web::get().to(route_oauth2::authorize::get))
            .route("/authorize", web::post().to(route_oauth2::authorize::post))
            .service(
                web::resource("/token")
                    .route(
                        web::post()
                            .guard(guard::Header(
                                "content-type",
                                "application/x-www-form-urlencoded",
                            ))
                            .to(route_oauth2::token::post_form),
                    )
                    .route(
                        web::post()
                            .guard(guard::Header("content-type", "application/json"))
                            .to(route_oauth2::token::post_json),
                    ),
            )
            .service(
                web::resource("/introspect")
                    .route(
                        web::post()
                            .guard(guard::Header(
                                "content-type",
                                "application/x-www-form-urlencoded",
                            ))
                            .to(route_oauth2::introspect::post_form),
                    )
                    .route(
                        web::post()
                            .guard(guard::Header("content-type", "application/json"))
                            .to(route_oauth2::introspect::post_json),
                    ),
            )
            .route("/redirect", web::get().to(route_oauth2::redirect::get))
    }

    fn public_v2_auth() -> web::Scope {
        web::scope("/auth")
            .route(
                "/password-reset",
                web::get().to(route_auth::password_reset::get),
            )
            .route(
                "/password-reset",
                web::post().to(route_auth::password_reset::post),
            )
            .route(
                "/password-update",
                web::get().to(route_auth::password_update::get),
            )
            .route(
                "/password-update",
                web::post().to(route_auth::password_update::post),
            )
            .route(
                "/email-update",
                web::get().to(route_auth::email_update::get),
            )
            .route(
                "/email-update",
                web::post().to(route_auth::email_update::post),
            )
            .route("/logout", web::get().to(route_auth::logout::get))
            .route("/register", web::get().to(route_auth::register::get))
            .route("/register", web::post().to(route_auth::register::post))
            .route("/delete", web::get().to(route_auth::delete::get))
            .route("/delete", web::post().to(route_auth::delete::post))
    }

    fn public_v2_audit() -> web::Scope {
        web::scope("/audit")
            .route("/create", web::post().to(route_audit::create::post))
            .route("/read", web::post().to(route_audit::read::post))
    }

    fn public_v2_csrf() -> web::Scope {
        web::scope("/csrf")
            .route("/create", web::post().to(route_csrf::create::post))
            .route("/verify", web::post().to(route_csrf::verify::post))
    }

    fn public_v2_client() -> web::Scope {
        web::scope("/client")
            .route("/read", web::post().to(route_client::read::post))
            .service(
                web::scope("/access")
                    .route("/read", web::post().to(route_client_access::read::post))
                    .route("/update", web::post().to(route_client_access::update::post))
                    .route("/delete", web::post().to(route_client_access::delete::post)),
            )
    }

    fn public_v2_user() -> web::Scope {
        web::scope("/user")
            .route("/create", web::post().to(route_user::create::post))
            .route("/read", web::post().to(route_user::read::post))
            .route("/update", web::post().to(route_user::update::post))
            .route("/delete", web::post().to(route_user::delete::post))
            .service(
                web::scope("/access")
                    .route("/read", web::post().to(route_user_access::read::post))
                    .route("/update", web::post().to(route_user_access::update::post))
                    .route("/delete", web::post().to(route_user_access::delete::post)),
            )
            .service(
                web::scope("/api-key")
                    .route("/create", web::post().to(route_user_api_key::create::post))
                    .route("/read", web::post().to(route_user_api_key::read::post))
                    .route("/update", web::post().to(route_user_api_key::update::post))
                    .route("/delete", web::post().to(route_user_api_key::delete::post))
                    .route("/verify", web::post().to(route_user_api_key::verify::post)),
            )
    }

    fn public_v2() -> web::Scope {
        web::scope("/v2")
            .service(Self::public_v2_oauth2())
            .service(Self::public_v2_auth())
            .service(Self::public_v2_audit())
            .service(Self::public_v2_csrf())
            .service(Self::public_v2_client())
            .service(Self::public_v2_user())
    }

    fn public_script() -> web::Scope {
        web::scope("/script")
            .route("/zxcvbn.js", web::get().to(route_script::zxcvbn_get))
            .route(
                "/password_strength.js",
                web::get().to(route_script::password_strength_get),
            )
    }

    fn public_well_known() -> web::Scope {
        web::scope("/.well-known")
            .route(
                "/openid-configuration",
                web::get().to(route_well_known::openid_configuration::get),
            )
            .route(
                "/change-password",
                web::get().to(route_well_known::change_password::get),
            )
    }

    pub fn public() -> web::Scope {
        web::scope("")
            .route("/ping", web::get().to(ping))
            .service(Self::public_v2())
            .service(Self::public_script())
            .service(Self::public_well_known())
    }

    pub fn private() -> web::Scope {
        web::scope("")
            .route("/ping", web::get().to(ping))
            .route("/liveness", web::get().to(ping))
            .route("/readiness", web::get().to(health))
            .route("/metrics", web::get().to(metrics))
    }
}

/// Returns response for uptime and kubernetes liveness tests
#[api_v2_operation(summary = "Server ping")]
async fn ping(server: Data<HttpServer>, req: HttpRequest) -> HttpResult<Json<String>> {
    server_request!(&server, &req, async { Ok(Json("ok".to_string())) })
}

/// Returns response for kubernetes readiness tests
#[api_v2_operation(summary = "Server health")]
async fn health(server: Data<HttpServer>, req: HttpRequest) -> HttpResult<Json<String>> {
    server_request!(&server, &req, async {
        server.readiness().await.map(|_| Json("ok".to_string()))
    })
}

/// Returns metrics in Prometheus exposition format
#[api_v2_operation(summary = "Server metrics in Prometheus exposition format")]
async fn metrics(server: Data<HttpServer>, req: HttpRequest) -> HttpResult<HttpResponse> {
    server_request!(&server, &req, async {
        // Using blocking doesn't appear to fix incrementing counters behaviour here
        // Is it because metrics are using sync instruments? is opentelemetry example wrong?
        let metrics = server.metrics.clone();
        let (format_type, buffer) = util::blocking(move || Ok(metrics.encode()))
            .await
            .map_err(HttpError::InternalServerError)?;

        Ok(actix_web::HttpResponse::build(http::StatusCode::OK)
            .content_type(format_type)
            .body(buffer))
    })
}
