## Key

### List [GET /v1/key]

List keys.

**Request**

Query parameters: `?gt=X&lt=Y&limit=Z`

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
- `meta.lt`: Less than ID, or null.
- `meta.limit`: Returned items limit.
- `data`: Array of IDs.

**Response [400, Bad Request]**

- Request query is invalid.

**Response [403, Forbidden]**

- Authorisation header is missing or invalid.

### Create [POST /v1/key]

Create key.

**Request**

```json
{
    "name": "Name",
    "user_id": 2
}
```

- `name`: Key name (required).
- `user_id`: User ID (required).

**Response [200, OK]**

```json
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

- `data.created_at`: Created time ISO 8601 timestamp.
- `data.updated_at`: Updated time ISO 8601 timestamp.
- `data.id`: Key ID.
- `data.name`: Key name.
- `data.value`: Key value.
- `data.service_id`: Key service ID relation.
- `data.user_id`: Key user ID relation or null.

**Response [400, Bad Request]**

- Request body is invalid.
- Key name is invalid.
- User ID is invalid or user does not exist.

**Response [403, Forbidden]**

- Authorisation header is invalid.

### Read [GET /v1/key/{id}]

Read key by ID.

**Response [200, OK]**

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

- `data.created_at`: Created time ISO 8601 timestamp.
- `data.updated_at`: Updated time ISO 8601 timestamp.
- `data.id`: Key ID.
- `data.name`: Key name.
- `data.value`: Key value.
- `data.service_id`: Key service ID relation.
- `data.user_id`: Key user ID relation or null.

**Response [403, Forbidden]**

- Authorisation header is invalid.

### Update [PATCH /v1/key/{id}]

Update key by ID.

**Request**

```json
{
	"name": "Key Name 2"
}
```

- `name`: Key name (optional).

**Response [200, OK]**

```json
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

- `data.created_at`: Created time ISO 8601 timestamp.
- `data.updated_at`: Updated time ISO 8601 timestamp.
- `data.id`: Key ID.
- `data.name`: Key name.
- `data.value`: Key value.
- `data.service_id`: Key service ID relation.
- `data.user_id`: Key user ID relation or null.

**Response [400, Bad Request]**

- Request body is invalid.
- Key name is invalid.

**Response [403, Forbidden]**

- Authorisation header is invalid.

### Delete [DELETE /v1/key/{id}]

Delete key by ID.

**Response [200, OK]**

**Response [400, Bad Request]**

- Key ID is invalid or key does not exist.

**Response [403, Forbidden]**

- Authorisation header is invalid.
