# Token

## Verify Token [/v1/auth/token/verify]

### Token [POST]

Verify user token.

#### Request

```JSON
{
    "token": "eyJ0e...Dlgu4",
}
```

- `token`: JWT authentication token for user (required).

#### Response [200, OK]

```JSON
{
    "data": {
        "user_id": 3,
        "token": "eyJ0e...Dlgu4",
        "token_expires": 1555957164
    }
}
```

##### Data

- `user_id`: User ID.
- `token`: JWT authentication token for user.
- `token_expires`: JWT expiry time, unix timestamp.

#### Response [400, Bad Request]

- Request body is invalid.
- Token is invalid.
- Token is not for authorised service.

#### Response [403, Forbidden]

- Authorisation header is invalid.

## Refresh Token [/v1/auth/token/refresh]

### Token [POST]

Refresh user token, creates new token with updated expiry time.

#### Request

```JSON
{
    "token": "eyJ0e...Dlgu4",
}
```

- `token`: JWT authentication token for user (required).

#### Response [200, OK]

```JSON
{
    "data": {
        "user_id": 3,
        "token": "eyJ0e...0ZT7k",
        "token_expires": 1555957470
    }
}
```

##### Data

- `user_id`: User ID.
- `token`: JWT authentication token for user.
- `token_expires`: JWT expiry time, unix timestamp.

#### Response [400, Bad Request]

- Request body is invalid.
- Token is invalid.
- Token is not for authorised service.

#### Response [403, Forbidden]

- Authorisation header is invalid.

## Revoke Token [/v1/auth/token/revoke]

### Token [POST]

Revoke user token, deletes associated key to prevent reuse.

#### Request

```JSON
{
    "token": "eyJ0e...Dlgu4",
}
```

- `token`: JWT authentication token for user (required).

#### Response [200, OK]

#### Response [400, Bad Request]

- Request body is invalid.
- Token is invalid.
- Token is not for authorised service.

#### Response [403, Forbidden]

- Authorisation header is invalid.
