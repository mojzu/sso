# Verify [POST /v1/auth/key/verify]

Verify user key.

## Request

```json
{
    "key": "5a044..4d37c",
}
```

- `key`: User key (required).

## Response [200, OK]

```json
{
    "data": {
        "user_id": 2,
        "key": "0e12c...e693f"
    }
}
```

### Data

- `user_id`: User ID.
- `key`: User key.

## Response [400, Bad Request]

- Request body is invalid.
- Key is invalid.
- Key is for a service.

## Response [403, Forbidden]

- Authorisation header is invalid.
