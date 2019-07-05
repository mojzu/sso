### Token

#### Refresh Token [POST /v1/auth/token/refresh]

Refresh user token, creates new token with updated expiry time.

**Request**

```json
{
  "token": "eyJ0e...Dlgu4"
}
```

- `token`: JWT authentication token for user (required).

**Response [200, OK]**

```json
{
  "data": {
    "user_id": 3,
    "token": "eyJ0e...0ZT7k",
    "token_expires": 1555957470
  }
}
```

- `data.user_id`: User ID.
- `data.token`: JWT authentication token for user.
- `data.token_expires`: JWT expiry time, unix timestamp.

**Response [400, Bad Request]**

- Request body is invalid.
- Token is invalid.
- Token is not for authorised service.

**Response [403, Forbidden]**

- Authorisation header is invalid.

#### Revoke Token [POST /v1/auth/token/revoke]

Revoke user token, deletes associated key to prevent reuse.

**Request**

```json
{
  "token": "eyJ0e...Dlgu4"
}
```

- `token`: JWT authentication token for user (required).

**Response [400, Bad Request]**

- Request body is invalid.
- Token is invalid.
- Token is not for authorised service.

**Response [403, Forbidden]**

- Authorisation header is invalid.

#### Verify Token [POST /v1/auth/token/verify]

Verify user token.

**Request**

```json
{
  "token": "eyJ0e...Dlgu4"
}
```

- `token`: JWT authentication token for user (required).

**Response [200, OK]**

```json
{
  "data": {
    "user_id": 3,
    "token": "eyJ0e...Dlgu4",
    "token_expires": 1555957164
  }
}
```

- `data.user_id`: User ID.
- `data.token`: JWT authentication token for user.
- `data.token_expires`: JWT expiry time, unix timestamp.

**Response [400, Bad Request]**

- Request body is invalid.
- Token is invalid.
- Token is not for authorised service.

**Response [403, Forbidden]**

- Authorisation header is invalid.
