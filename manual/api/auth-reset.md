# Reset

## Reset Password [/v1/auth/reset/password]

### Email Request [POST]

Reset password request for email address.

#### Request

```JSON
{
    "email": "user@example.com"
}
```

- `email`: User email address (required).

#### Response [200, OK]

#### Response [400, Bad Request]

- Request body is invalid.
- User email address is invalid or unknown.
- User is not assigned key for service.
- User password is null.
- Unable to send password reset email.

#### Response [403, Forbidden]

- Authorisation header is invalid.

## Reset Password Confirm [/v1/auth/reset/password/confirm]

### Token Confirm [POST]

Confirm reset password request.

#### Request

```JSON
{
    "token": "eyJ0e...6eEvY",
    "password": "guest"
}
```

- `token`: JWT reset authentication token for user (required).
- `password`: User password (required).

#### Response [200, OK]

```JSON
{
    "meta": {
        "password_strength": 2,
        "password_pwned": false
    }
}
```

##### Meta

- `password_strength`: Password strength score by `zxcvbn`, null if scoring failed.
- `password_pwned`: True if password detected by `haveibeenpwned.com`, null if request failed or feature disabled.

#### Response [400, Bad Request]

- Request body is invalid.
- Token is invalid or has been used.
- User password is invalid.
- User is not assigned key for service.

#### Response [403, Forbidden]

- Authorisation header is invalid.
