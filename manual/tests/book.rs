use manual::*;

#[test]
fn api_ping() {
    let client = create_client();
    ping_server(&client);
}

#[test]
fn guide_api_key() {
    let mut client = create_client();
    let (service, service_key) = create_service_key(&client);
    let user_email = create_user_email();

    client.options.set_authorisation(&service_key.value);
    let user = create_user(&client, "User Name", &user_email, true, None);
    let user_key = create_user_key(&client, "Key Name", service.id, user.id);

    verify_user_key(&client, &user_key);
}

#[test]
fn guide_login() {
    let mut client = create_client();
    let (service, service_key) = create_service_key(&client);
    let user_email = create_user_email();

    client.options.set_authorisation(&service_key.value);
    let user = create_user(&client, "User Name", &user_email, true, Some("guest"));
    let _user_key = create_user_key(&client, "Key Name", service.id, user.id);

    let user_token = local_login(&client, user.id, &user_email, "guest");
    verify_user_token(&client, &user_token);
}

#[test]
fn guide_oauth2_login() {
    let mut client = create_client();
    let (service, service_key) = create_service_key(&client);
    let user_email = create_user_email();

    client.options.set_authorisation(&service_key.value);
    let user = create_user(&client, "User Name", &user_email, true, Some("guest"));
    let _user_key = create_user_key(&client, "Key Name", service.id, user.id);

    microsoft_oauth2_request(&client);
}

#[test]
fn guide_reset_password() {
    let mut client = create_client();
    let (service, service_key) = create_service_key(&client);
    let user_email = create_user_email();

    client.options.set_authorisation(&service_key.value);
    let user = create_user(&client, "User Name", &user_email, true, Some("guest"));
    let _user_key = create_user_key(&client, "Key Name", service.id, user.id);

    local_password_reset(&client, &user_email);
}
