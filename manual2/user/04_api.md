# API

## Ping [GET /v1/ping]

Uptime test route, authentication is not required for this route.

**Response [200, OK]**

```json
"pong"
```

## Authentication

### GitHub Provider

#### OAuth2 Request [POST /v1/auth/provider/github/oauth2]

Request an OAuth2 authentication redirect URL.

**Response [200, OK]**

```json
{
  "url": "https://github.com/login/oauth/authorize?...JAMBQ"
}
```

- `url`: Authentication redirect URL.

**Response [403, Forbidden]**

- Authorisation header is invalid.

#### OAuth2 Callback [GET /v1/auth/provider/github/oauth2]

User redirected here by GitHub OAuth2 server after successful authentication.

### Microsoft Provider

#### OAuth2 Request [POST /v1/auth/provider/microsoft/oauth2]

Request an OAuth2 authentication redirect URL.

**Response [200, OK]**

```json
{
  "url": "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?...aRgyE"
}
```

- `url`: Authentication redirect URL.

**Response [403, Forbidden]**

- Authorisation header is invalid.

#### OAuth2 Callback [GET /v1/auth/provider/microsoft/oauth2]

User redirected here by Microsoft OAuth2 server after successful authentication.
