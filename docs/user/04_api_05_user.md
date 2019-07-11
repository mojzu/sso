## User

### List [GET /v1/user]

List users.

**Request**

```
?gt=X&lt=Y&limit=Z
```

- `gt`: Greater than ID (optional).
- `lt`: Less than ID (optional).
- `limit`: Limit number of returned items (optional).

**Response [200, OK]**

```json
{
    "meta": {
        "gt": 0,
        "lt": null,
        "limit": 10
    },
    "data": [
        1,
        ...
    ]
}
```

- `meta.gt`: Greater than ID, or null.
- `meta.lt`: Greater than ID, or null.
- `meta.limit`: Returned items limit.
- `data`: Array of IDs.

**Response [400, Bad Request]**

- Request query is invalid.

**Response [403, Forbidden]**

- Authorisation header is invalid.

### Create [POST /v1/user]

Create user.

**Request**

```json
{
  "is_enabled": true,
  "name": "User Name",
  "email": "user@example.com",
  "password": "guest"
}
```

- `is_enabled`: User is enabled flag (required).
- `name`: User name (required).
- `email`: User email address, must be unique (required).
- `password`: User password, optional.

**Response [200, OK]**

```json
{
  "meta": {
    "password_strength": 0,
    "password_pwned": true
  },
  "data": {
    "created_at": "2019...04Z",
    "updated_at": "2019...04Z",
    "id": 10,
    "is_enabled": true,
    "name": "User Name",
    "email": "user@example.com"
  }
}
```

- `meta.password_strength`: Password strength score by `zxcvbn`, null if scoring failed or password was not provided.
- `meta.password_pwned`: True if password detected by `haveibeenpwned.com`, null if request failed, feature disabled or password was not provided.
- `data.created_at`: Created time ISO 8601 timestamp.
- `data.updated_at`: Updated time ISO 8601 timestamp.
- `data.id`: User ID.
- `data.is_enabled`: User is enabled flag.
- `data.name`: User name.
- `data.email`: User email address.

**Response [400, Bad Request]**

- Request body is invalid.
- User name is invalid.
- User email address is invalid or not unique.
- User password is invalid.

**Response [403, Forbidden]**

- Authorisation header is invalid.

### Read [GET /v1/user/{id}]

Read user by ID.

**Response [200, OK]**

```json
{
    "data": {
        "created_at": "2019...46Z",
        "updated_at": "2019...46Z",
        "id": 1,
        "name": "User Name",
        "email": "user@example.com"
    }
}
```

- `data.created_at`: Created time ISO 8601 timestamp.
- `data.updated_at`: Updated time ISO 8601 timestamp.
- `data.id`: User ID.
- `data.name`: User name.
- `data.email`: User email address.

**Response [403, Forbidden]**

- Authorisation header is invalid.

### Update [PATCH /v1/user/{id}]

Update user by ID.

**Request**

```json
{
	"name": "User Name 2"
}
```

- `name`: User name (optional).

**Response [200, OK]**

```json
{
    "data": {
        "created_at": "2019...46Z",
        "updated_at": "2019...95Z",
        "id": 1,
        "name": "User Name 2",
        "email": "user@example.com"
    }
}
```

- `data.created_at`: Created time ISO 8601 timestamp.
- `data.updated_at`: Updated time ISO 8601 timestamp.
- `data.id`: User ID.
- `data.name`: User name.
- `data.email`: User email address.

**Response [400, Bad Request]**

- Request body is invalid.
- User name is invalid.

**Response [403, Forbidden]**

- Authorisation header is invalid.

### Delete [DELETE /v1/user/{id}]

Delete user by ID.

**Response [200, OK]**

**Response [400, Bad Request]**

- User ID is invalid or user does not exist.

**Response [403, Forbidden]**

- Authorisation header is invalid.
