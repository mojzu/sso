use actix_http::HttpService;
use actix_http_test::TestServer;
use actix_web::http::{header, StatusCode};
use actix_web::{web, App, HttpResponse};
use mr_auth::api;
use mr_auth::models::{AuthKey, AuthService, AuthUser};

pub fn app() -> (mr_auth::db::Db, actix_http_test::TestServerRuntime) {
    let config = api::ApiConfig::new("localhost:9001".to_owned());
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let db = mr_auth::db::Db::new(&db_url);
    let db_app = db.clone();

    let server = TestServer::new(move || {
        HttpService::new(
            App::new()
                .data(api::ApiData::new(config.clone(), db_app.clone()))
                .wrap(api::ApiIdentityPolicy::identity_service())
                .configure(api::app)
                .default_service(web::route().to(|| HttpResponse::MethodNotAllowed())),
        )
    });

    (db, server)
}

pub fn app_post<T: Into<actix_http::body::Body>>(
    app: &mut actix_http_test::TestServerRuntime,
    uri: &str,
    authorisation: Option<&str>,
    payload: T,
) -> (StatusCode, usize, Vec<u8>) {
    let req = app
        .post(uri)
        .header(header::CONTENT_TYPE, header::ContentType::json());
    let req = match authorisation {
        Some(authorisation) => req.header(header::AUTHORIZATION, authorisation),
        None => req,
    };
    let req = req.send_body(payload);
    let mut res = app.block_on(req).unwrap();

    let status_code = res.status();
    let content_length = res
        .headers()
        .get(header::CONTENT_LENGTH)
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let bytes = app.block_on(res.body()).unwrap();

    (status_code, content_length, bytes.to_vec())
}

pub fn service_key(db: &mr_auth::db::Db) -> (AuthService, AuthKey) {
    let random = uuid::Uuid::new_v4().to_simple().to_string();
    db.init(&random, &random).unwrap()
}

pub fn user_key(
    db: &mr_auth::db::Db,
    service: &AuthService,
    password: Option<&str>,
) -> (AuthUser, AuthKey) {
    let random = uuid::Uuid::new_v4().to_simple().to_string();
    let email = format!("{}@example.com", random);
    let user = db.user_create("User Name", &email, password).unwrap();
    let key = db
        .key_create("Key Name", service.service_id, Some(user.user_id))
        .unwrap();
    (user, key)
}
