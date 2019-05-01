# API

HTTP endpoints are available at `$SERVER_BIND`.

Requests are authenticated using `Authorization: $key_value` header. Service keys can be created via command line.

```Shell
$ ark_auth init $service_name $service_url
```

## Ping [/v1/ping]

### Ping [GET]

Uptime test route, authentication is not required for this route.

#### Response [200, OK]

```JSON
"pong"
```
