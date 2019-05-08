# OAuth2

## GitHub [/v1/auth/oauth2/github]

### Request [POST]

Request a authentication redirect URL.

#### Response [200, OK]

```JSON
{
    "url": "https://github.com/login/oauth/authorize?...JAMBQ"
}
```

- `url`: Authentication redirect URL.

#### Response [403, Forbidden]

- Authorisation header is invalid.

### Callback [GET]

Used by GitHub OAuth2 provider.

## Microsoft [/v1/auth/oauth2/microsoft]

### Request [POST]

Request a authentication redirect URL.

#### Response [200, OK]

```JSON
{
    "url": "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?...aRgyE"
}
```

- `url`: Authentication redirect URL.

#### Response [403, Forbidden]

- Authorisation header is invalid.

### Callback [GET]

Used by Microsoft OAuth2 provider.