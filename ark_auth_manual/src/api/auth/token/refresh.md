# Refresh Token [POST /v1/auth/token/refresh]

Refresh user token, creates new token with updated expiry time.

## Request

```json
{
    "token": "eyJ0e...Dlgu4",
}
```

- `token`: JWT authentication token for user (required).

## Response [200, OK]

```json
{
    "data": {
        "user_id": 3,
        "token": "eyJ0e...0ZT7k",
        "token_expires": 1555957470
    }
}
```

### Data

- `user_id`: User ID.
- `token`: JWT authentication token for user.
- `token_expires`: JWT expiry time, unix timestamp.

## Response [400, Bad Request]

- Request body is invalid.
- Token is invalid.
- Token is not for authorised service.

#### Response [403, Forbidden]

- Authorisation header is invalid.
