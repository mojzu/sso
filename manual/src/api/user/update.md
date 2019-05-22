# Update [PATCH /v1/user/{id}]

Update user by ID.

## Request

```json
{
	"name": "User Name 2"
}
```

- `name`: User name (optional).

## Response [200, OK]

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

### Data

- `created_at`: Created time ISO 8601 timestamp.
- `updated_at`: Updated time ISO 8601 timestamp.
- `id`: User ID.
- `name`: User name.
- `email`: User email address.

## Response [400, Bad Request]

- Request body is invalid.
- User name is invalid.

## Response [403, Forbidden]

- Authorisation header is invalid.
