### Local Provider

#### Login [POST /v1/auth/provider/local/login]

Login with email address and password.

**Request**

```json
{
  "email": "user@example.com",
  "password": "guest"
}
```

- `email`: User email address (required).
- `password`: User password (required).

**Response [200, OK]**

```json
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

- `meta.password_strength`: Password strength score by `zxcvbn`, null if scoring failed.
- `meta.password_pwned`: True if password detected by `haveibeenpwned.com`, null if request failed or feature disabled.
- `data.user_id`: User ID.
- `data.token`: JWT authentication token for user.
- `data.token_expires`: JWT expiry time, unix timestamp.

**Response [400, Bad Request]**

- Request body is invalid.
- User email address is invalid or unknown.
- User password is invalid or incorrect or null.
- User is not assigned key for service.

**Response [403, Forbidden]**

- Authorisation header is invalid.

#### Reset Password [POST /v1/auth/provider/local/reset-password]

Reset password request for email address.

**Request**

```json
{
  "email": "user@example.com"
}
```

- `email`: User email address (required).

**Response [200, OK]**

**Response [400, Bad Request]**

- Request body is invalid.
- User email address is invalid or unknown.
- User is not assigned key for service.
- User password is null.
- Unable to send password reset email.

**Response [403, Forbidden]**

- Authorisation header is invalid.

#### Reset Password Confirm [POST /v1/auth/provider/local/reset-password/confirm]

Confirm reset password request.

**Request**

```json
{
  "token": "eyJ0e...6eEvY",
  "password": "guest"
}
```

- `token`: JWT reset authentication token for user (required).
- `password`: User password (required).

**Response [200, OK]**

```json
{
  "meta": {
    "password_strength": 2,
    "password_pwned": false
  }
}
```

- `meta.password_strength`: Password strength score by `zxcvbn`, null if scoring failed.
- `meta.password_pwned`: True if password detected by `haveibeenpwned.com`, null if request failed or feature disabled.

**Response [400, Bad Request]**

- Request body is invalid.
- Token is invalid or has been used.
- User password is invalid.
- User is not assigned key for service.

**Response [403, Forbidden]**

- Authorisation header is invalid.

#### Update Email [POST /v1/auth/provider/local/update-email]

Update email request for user.

#### Update Email Revoke [POST /v1/auth/provider/local/update-email/revoke]

Revoke update email request for user.

#### Update Password [POST /v1/auth/provider/local/update-password]

Update password request for user.

#### Update Password Revoke [POST /v1/auth/provider/local/update-password/revoke]

Revoke update password request for user.
