use manual::*;

#[test]
fn guide_reset_password() {
    let mut client = create_client();
    let (_service, service_key) = create_service_key(&client);
    let user_email = create_user_email();

    client = client.set_authorisation(&service_key.value);
    let user = create_user(&client, "User Name", &user_email, true, Some("guest"));
    let _user_key = create_user_key(&client, "Key Name", user.id);

    request_password_reset(&client, &user_email);
}
