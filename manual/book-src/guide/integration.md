# Integration

A service integrating with Ark Auth can provide the following HTTPS endpoints.

```shell
GET $service_url?token=$token
```

User is redirected to this URL after successful authentication by an OAuth2 provider. The service can verify the `token` query parameter to authenticate the user and their requests.

```shell
GET $service_url?email=$email&reset_password_token=$token
```

User requests this URL by clicking reset password email link. The service can take password input from the user, and then make a reset password confirm request with the `reset_password_token` query parameter.

```shell
GET $service_url?email=$email&old_email=$email&update_email_token=$token
```

User requests this URL by clicking update email link. The service can present the user information on securing their account, and then make an update email revoke request with the `update_email_token` query parameter. This will disable all user keys.

```shell
GET $service_url?email=$email&update_password_token=$token
```

User requests this URL by clicking update password link. The service can present the user information on securing their account, and then make an update password revoke request with the `update_password_token` query parameter. This will disable all user keys.
