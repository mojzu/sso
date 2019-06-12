# Integration

A service integrating with Ark Auth can provide the following HTTPS endpoints.

```shell
GET $service_url?token=$token
```

User is redirected to this URL after successful authentication by an OAuth2 provider. The service can verify the token query parameter to authenticate the user and their requests.

```shell
GET $service_url?email=$user_email&reset_password_token=$token
```

User requests this URL by clicking reset password email link. The service can take password input from the user and the make a reset password confirm request with the reset password token query parameter.
