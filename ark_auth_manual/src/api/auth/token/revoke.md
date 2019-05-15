# Revoke Token [POST /v1/auth/token/revoke]

Revoke user token, deletes associated key to prevent reuse.

## Request

```json
{
    "token": "eyJ0e...Dlgu4",
}
```

- `token`: JWT authentication token for user (required).

## Response [200, OK]

## Response [400, Bad Request]

- Request body is invalid.
- Token is invalid.
- Token is not for authorised service.

## Response [403, Forbidden]

- Authorisation header is invalid.
