## Service

#### List [GET /v1/service]

#### Create [POST /v1/service]

#### Read [GET /v1/service/{id}]

Read service by ID.

**Response [200, OK]**

```json
{
  "data": {
    "created_at": "2019...90Z",
    "updated_at": "2019...90Z",
    "id": 1,
    "name": "Service Name",
    "url": "https://..."
  }
}
```

- `data.created_at`: Created time ISO 8601 timestamp.
- `data.updated_at`: Updated time ISO 8601 timestamp.
- `data.id`: Service ID.
- `data.name`: Service name.
- `data.url`: Service URL.

**Response [403, Forbidden]**

- Authorisation header is invalid.

#### Update [PATCH /v1/service/{id}]

TODO(manual): Finish, improve manual API documentation.

#### Delete [DELETE /v1/service/{id}]
