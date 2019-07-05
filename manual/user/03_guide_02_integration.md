## Integration

The following diagram illustrates how services and ark_auth integrate to authenticate user requests.

![User request verification](manual/asset/integration.svg)

1. User with token or key acquired by authentication method sends HTTP request to service.
2. Service sends HTTP request to ark_auth with its own service key, and the users token or key.
3. ark_auth authenticates service using the service key, and verifies user token or key.
4. If authenticated/verified, service handles request and sends HTTP response to user.
5. User handles HTTP response.

**Callbacks**

A service integrating with ark_auth can provide the following HTTPS endpoints as callbacks.

```shell
GET $service_url?user_id=$id&access_token=$token&access_token_expires=$token_expires&refresh_token=$token&refresh_token_expires=$token_expires
```

User is redirected to this URL after successful authentication by an OAuth2 provider. The service can verify the `access_token` query parameter to authenticate the user and their requests.

```shell
GET $service_url?email=$email&reset_password_token=$token
```

User requests this URL by clicking reset password email link. The service can take password input from the user, and then make a reset password confirm request with the `reset_password_token` query parameter.

```shell
GET $service_url?email=$email&old_email=$email&update_email_token=$token
```

User requests this URL by clicking update email revoke link. The service can present the user information on securing their account, and then make an update email revoke request with the `update_email_token` query parameter. This will disable the user and all user keys.

```shell
GET $service_url?email=$email&update_password_token=$token
```

User requests this URL by clicking update password revoke link. The service can present the user information on securing their account, and then make an update password revoke request with the `update_password_token` query parameter. This will disable the user and all user keys.
