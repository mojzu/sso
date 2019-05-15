# Reset Password [POST /v1/auth/reset/password]

Reset password request for email address.

## Request

```json
{
    "email": "user@example.com"
}
```

- `email`: User email address (required).

## Response [200, OK]

## Response [400, Bad Request]

- Request body is invalid.
- User email address is invalid or unknown.
- User is not assigned key for service.
- User password is null.
- Unable to send password reset email.

## Response [403, Forbidden]

- Authorisation header is invalid.
