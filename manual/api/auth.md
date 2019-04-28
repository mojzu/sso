# Authentication

## Login [/v1/auth/login]

### Email and Password [POST]

Login with email address and password.

#### Request

```JSON
{
    "email": "user@example.com",
    "password": "guest"
}
```

- `email`: User email address (required).
- `password`: User password (required).

#### Response [200, OK]

```JSON
{
    "meta": {
        "password_strength": 2,
        "password_pwned": false
    },
    "data": {
        "user_id": 1,
        "token": "eyJ0e...6eEvY",
        "token_expires": 1555881550
    }
}
```

##### Meta

- `password_strength`: Password strength score by `zxcvbn`, null if scoring failed.
- `password_pwned`: True if password detected by `haveibeenpwned.com`, null if request failed or feature disabled.

##### Data

- `user_id`: User ID.
- `token`: JWT authentication token for user.
- `token_expires`: JWT expiry time, unix timestamp.

#### Response [400, Bad Request]

- Request body is invalid.
- User email address is invalid or unknown.
- User password is invalid or incorrect or null.
- User is not assigned key for service.

#### Response [403, Forbidden]

- Authorisation header is invalid.
