# Read [GET /v1/key/{id}]

Read key by ID.

## Response [200, OK]

```json
{
    "data": {
        "created_at": "2019...90Z",
        "updated_at": "2019...90Z",
        "id": 1,
        "name": "Key Name",
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

## Response [403, Forbidden]

- Authorisation header is invalid.
