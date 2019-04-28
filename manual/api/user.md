# User

## Collection [/v1/user]

### Read Many [GET]

TODO(doc)

### Create One [POST]

Create one user.

#### Request

```JSON
{
    "name": "User Name",
    "email": "user@example.com",
    "password": "guest"
}
```

- `name`: User name (required).
- `email`: User email address, must be unique (required).
- `password`: User password, optional.

#### Response [200, OK]

```JSON
{
    "meta": {
        "password_strength": 0,
        "password_pwned": true
    },
    "data": {
        "created_at": "2019...04Z",
        "updated_at": "2019...04Z",
        "id": 10,
        "name": "User Name",
        "email": "user@example.com"
    }
}
```

##### Meta

- `password_strength`: Password strength score by `zxcvbn`, null if scoring failed or password was not provided.
- `password_pwned`: True if password detected by `haveibeenpwned.com`, null if request failed, feature disabled or password was not provided.

##### Data

- `created_at`: Created time ISO 8601 timestamp.
- `updated_at`: Updated time ISO 8601 timestamp.
- `id`: User ID.
- `name`: User name.
- `email`: User email address.

#### Response [400, Bad Request]

- Request body is invalid.
- User name is invalid.
- User email address is invalid or not unique.
- User password is invalid.

#### Response [403, Forbidden]

- Authorisation header is invalid.

## User [/v1/user/{id}]

### Read One [GET]

TODO(doc)

### Update One [PATCH]

TODO(doc)

### Delete One [DELETE]

TODO(doc)
