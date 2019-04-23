# Key

## Collection [/v1/key]

### Read Many [GET]

Read many keys.

#### Request

```
?offset=X&limit=Y&order=Z
```

-   `offset`: ID offset (optional).
-   `limit`: Limit number of returned items (optional).
-   `order`: ID order, `asc` or `desc` (optional).

#### Response [200, OK]

```JSON
{
    "data": [
        ...
    ]
}
```

-   `data`: Array of read items.

#### Response [400, Bad Request]

-   Request query is invalid.

#### Response [403, Forbidden]

-   Authorisation header is invalid.

### Create One [POST]

Create one key.

#### Request

```JSON
{
    "name": "Name",
    "user_id": 2
}
```

-   `name`: Key name (required).
-   `user_id`: User ID (required).

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

### Read One [GET]

TODO(doc)

### Update One [PATCH]

TODO(doc)

### Delete One [DELETE]

TODO(doc)
