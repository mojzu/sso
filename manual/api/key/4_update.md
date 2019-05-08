# Update [PATCH /v1/key/{id}]

Update key by ID.

## Request

```JSON
{
	"name": "Key Name 2"
}
```

- `name`: Key name (optional).

## Response [200, OK]

```JSON
{
    "data": {
        "created_at": "2019...90Z",
        "updated_at": "2019...28Z",
        "id": 1,
        "name": "Key Name 2",
        "value": "667...35c",
        "service_id": 1,
        "user_id": null
    }
}
```

### Data

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

## Response [403, Forbidden]

- Authorisation header is invalid.
