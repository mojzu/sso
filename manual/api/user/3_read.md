# Read [GET /v1/user/{id}]

Read user by ID.

## Response [200, OK]

```JSON
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

### Data

- `created_at`: Created time ISO 8601 timestamp.
- `updated_at`: Updated time ISO 8601 timestamp.
- `id`: User ID.
- `name`: User name.
- `email`: User email address.

## Response [403, Forbidden]

- Authorisation header is invalid.
