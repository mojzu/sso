# Create [POST /v1/key]

Create key.

## Request

```JSON
{
    "name": "Name",
    "user_id": 2
}
```

- `name`: Key name (required).
- `user_id`: User ID (required).

## Response [200, OK]

```JSON
{
    "data": {
        "created_at": "2019...92Z",
        "updated_at": "2019...92Z",
        "id": 2,
        "name": "Key Name",
        "value": "84b...dc6",
        "service_id": 1,
        "user_id": 1
    }
}
```

### Data

Created key.

- `created_at`: Created time ISO 8601 timestamp.
- `updated_at`: Updated time ISO 8601 timestamp.
- `id`: Key ID.
- `name`: Key name.
- `value`: Key value.
- `service_id`: Key service ID relation.
- `user_id`: Key user ID relation or null.

## Response [400, Bad Request]

- Request body is invalid.
- Key name is invalid.
- User ID is invalid or user does not exist.

## Response [403, Forbidden]

- Authorisation header is invalid.
