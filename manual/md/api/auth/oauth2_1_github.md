# GitHub Request [POST /v1/auth/oauth2/github]

Request a authentication redirect URL.

## Response [200, OK]

```json
{
  "url": "https://github.com/login/oauth/authorize?...JAMBQ"
}
```

- `url`: Authentication redirect URL.

## Response [403, Forbidden]

- Authorisation header is invalid.

# GitHub Callback [GET /v1/auth/oauth2/github]

Used by GitHub OAuth2 provider.
