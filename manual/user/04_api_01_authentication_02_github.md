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
