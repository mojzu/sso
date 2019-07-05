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
