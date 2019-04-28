# Key

## Verify Key [/v1/auth/key/verify]

### Key [POST]

Verify user key.

#### Request

```JSON
{
    "key": "5a044..4d37c",
}
```

- `key`: User key (required).

#### Response [200, OK]

```JSON
{
    "data": {
        "user_id": 2,
        "key": "0e12c...e693f"
    }
}
```

##### Data

- `user_id`: User ID.
- `key`: User key.

#### Response [400, Bad Request]

- Request body is invalid.
- Key is invalid.
- Key is for a service.

#### Response [403, Forbidden]

- Authorisation header is invalid.

## Revoke Key [/v1/auth/key/revoke]

### Key [POST]

Revoke a user key, any associated tokens will become invalid.

#### Request

```JSON
{
    "key": "5a044..4d37c",
}
```

- `key`: User key (required).

#### Response [200, OK]

#### Response [400, Bad Request]

- Request body is invalid.
- Key is invalid.
- Key is for a service.

#### Response [403, Forbidden]

- Authorisation header is invalid.
