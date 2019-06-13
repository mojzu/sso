# Microsoft Request [POST /v1/auth/provider/microsoft/oauth2]

Request a authentication redirect URL.

## Response [200, OK]

```json
{
  "url": "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?...aRgyE"
}
```

- `url`: Authentication redirect URL.

## Response [403, Forbidden]

- Authorisation header is invalid.

# Microsoft Callback [GET /v1/auth/provider/microsoft/oauth2]

Used by Microsoft OAuth2 provider.
