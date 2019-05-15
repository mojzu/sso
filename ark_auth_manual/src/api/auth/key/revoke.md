# Revoke [POST /v1/auth/key/revoke]

Revoke a user key, any associated tokens will become invalid.

## Request

```json
{
    "key": "5a044..4d37c",
}
```

- `key`: User key (required).

## Response [200, OK]

## Response [400, Bad Request]

- Request body is invalid.
- Key is invalid.
- Key is for a service.

## Response [403, Forbidden]

- Authorisation header is invalid.
