# List [GET /v1/key]

List keys.

## Request

```
?gt=X&lt=Y&limit=Z
```

- `gt`: Greater than ID (optional).
- `lt`: Less than ID (optional).
- `limit`: Limit number of returned items (optional).

## Response [200, OK]

```JSON
{
    "meta": {
        "gt": 0,
        "lt": null,
        "limit": 10
    },
    "data": [
        {
            "created_at": "2019...90Z",
            "updated_at": "2019...90Z",
            "id": 1,
            "name": "Key Name",
            "value": "667...35c",
            "service_id": 1,
            "user_id": null
        },
        ...
    ]
}
```

### Meta

- `gt`: Greater than ID, or null.
- `lt`: Greater than ID, or null.
- `limit`: Returned items limit.

### Data

Array of read items.

- `created_at`: Created time ISO 8601 timestamp.
- `updated_at`: Updated time ISO 8601 timestamp.
- `id`: Key ID.
- `name`: Key name.
- `value`: Key value.
- `service_id`: Key service ID relation.
- `user_id`: Key user ID relation or null.

## Response [400, Bad Request]

- Request query is invalid.

## Response [403, Forbidden]

- Authorisation header is invalid.
