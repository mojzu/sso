## 0.5.0 (?)

**Added**

- Added TOTP validation route.
- Added key flags: `allow_key`, `allow_token`, `allow_totp`.
- Added user fields: `locale`, `timezone`.

**Changed**

- Renamed project `mz_auth`, ark was taken by various products and crates.
- Code consistency improvements.
- Renamed audit `path` to `type` string.
- Replaced `serde_urlencoded` with `serde_qs` for more advanced query string support.
- Improved audit metric collection efficiency.
- Changed OAuth2 interface so callbacks go to service.

**Fixed**
