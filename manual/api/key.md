# Key

## Collection [/v1/key]

### List [GET]

List keys.

#### Request

```
?gt=X&lt=Y&limit=Z
```

- `gt`: Greater than ID (optional).
- `lt`: Less than ID (optional).
- `limit`: Limit number of returned items (optional).

#### Response [200, OK]

```JSON
{
    "meta": {
        "gt": 0,
        "lt": null,
        "limit": 10
    },
    "data": [
        ...
    ]
}
```

##### Meta

- `gt`: Greater than ID, or null.
- `lt`: Greater than ID, or null.
- `limit`: Returned items limit.

##### Data

Array of read items.

#### Response [400, Bad Request]

- Request query is invalid.

#### Response [403, Forbidden]

- Authorisation header is invalid.

### Create [POST]

Create key.

#### Request

```JSON
{
    "name": "Name",
    "user_id": 2
}
```

- `name`: Key name (required).
- `user_id`: User ID (required).

#### Response [200, OK]

```JSON
{
    "created_at": "2019...304Z",
    "id": 39,
    "name": "Name",
    "value": "455c7...c8630",
    "service_id": 26,
    "user_id": 10
}
```

## Key [/v1/key/{id}]

### Read [GET]

TODO(doc)

### Update [PATCH]

TODO(doc)

### Delete [DELETE]

TODO(doc)
