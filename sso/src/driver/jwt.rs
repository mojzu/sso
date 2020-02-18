use crate::{
    CsrfCreate, DriverError, DriverResult, KeyWithValue, Postgres, Service, User, UserToken,
};
use jsonwebtoken::{
    dangerous_unsafe_decode, decode, encode, DecodingKey, EncodingKey, Header, Validation,
};
use uuid::Uuid;

/// JSON web token maximum length validation.
pub const MAX_JWT: usize = 1000;

/// JSON web token claims types.
#[derive(Debug)]
pub enum JwtClaimsType {
    /// Access tokens used to verify user access to a service.
    AccessToken,
    /// Refresh tokens used to refresh user access tokens.
    RefreshToken,
    /// Register tokens used to verify user registration.
    RegisterToken,
    /// Reset password tokens used to verify user password resets.
    ResetPasswordToken,
    /// Revoke tokens used to revoke user tokens and keys.
    RevokeToken,
}

impl JwtClaimsType {
    /// Returns i64 representation of claims type.
    pub fn to_i64(&self) -> i64 {
        match self {
            JwtClaimsType::AccessToken => 0,
            JwtClaimsType::RefreshToken => 1,
            JwtClaimsType::RegisterToken => 2,
            JwtClaimsType::ResetPasswordToken => 3,
            JwtClaimsType::RevokeToken => 4,
        }
    }

    /// Returns claims type or error if not valid.
    pub fn from_i64(value: i64) -> DriverResult<Self> {
        match value {
            0 => Ok(JwtClaimsType::AccessToken),
            1 => Ok(JwtClaimsType::RefreshToken),
            2 => Ok(JwtClaimsType::RegisterToken),
            3 => Ok(JwtClaimsType::ResetPasswordToken),
            4 => Ok(JwtClaimsType::RevokeToken),
            _ => Err(DriverError::JwtClaimsTypeInvalid),
        }
    }
}

/// JSON web token claims.
#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    iss: String,
    sub: String,
    exp: i64,
    #[serde(rename = "x-type")]
    x_type: i64,
    #[serde(rename = "x-csrf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    x_csrf: Option<String>,
}

impl JwtClaims {
    pub fn new<IS, SU>(iss: IS, sub: SU, exp: i64, x_type: JwtClaimsType) -> Self
    where
        IS: Into<String>,
        SU: Into<String>,
    {
        let dt = chrono::Utc::now();
        let exp = dt.timestamp() + exp;
        JwtClaims {
            iss: iss.into(),
            sub: sub.into(),
            exp,
            x_type: x_type.to_i64(),
            x_csrf: None,
        }
    }

    pub fn new_csrf<IS, SU, CS>(
        iss: IS,
        sub: SU,
        exp: i64,
        x_type: JwtClaimsType,
        x_csrf: CS,
    ) -> Self
    where
        IS: Into<String>,
        SU: Into<String>,
        CS: Into<String>,
    {
        let mut claims = JwtClaims::new(iss, sub, exp, x_type);
        claims.x_csrf = Some(x_csrf.into());
        claims
    }

    pub fn validation<IS, SU>(iss: IS, sub: SU) -> Validation
    where
        IS: Into<String>,
        SU: Into<String>,
    {
        Validation {
            iss: Some(iss.into()),
            sub: Some(sub.into()),
            ..Validation::default()
        }
    }
}

/// JSON web token functions.
#[derive(Debug)]
pub struct Jwt;

impl Jwt {
    /// Encode a token, returns token and expiry time.
    pub fn encode_token(
        service_id: Uuid,
        user_id: Uuid,
        x_type: JwtClaimsType,
        key_value: &str,
        exp: i64,
    ) -> DriverResult<(String, i64)> {
        let claims = JwtClaims::new(service_id.to_string(), user_id.to_string(), exp, x_type);
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(key_value.as_bytes()),
        )
        .map_err(DriverError::Jsonwebtoken)?;
        Ok((token, claims.exp))
    }

    /// Encode a CSRF token, returns token and expiry time.
    pub fn encode_token_csrf(
        service_id: Uuid,
        user_id: Uuid,
        x_type: JwtClaimsType,
        x_csrf: &str,
        key_value: &str,
        exp: i64,
    ) -> DriverResult<(String, i64)> {
        let claims = JwtClaims::new_csrf(
            service_id.to_string(),
            user_id.to_string(),
            exp,
            x_type,
            x_csrf,
        );
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(key_value.as_bytes()),
        )
        .map_err(DriverError::Jsonwebtoken)?;
        Ok((token, claims.exp))
    }

    /// Safely decodes a token, returns expiry time and optional CSRF key.
    pub fn decode_token(
        service_id: Uuid,
        user_id: Uuid,
        x_type: JwtClaimsType,
        key_value: &str,
        token: &str,
    ) -> DriverResult<(i64, Option<String>)> {
        let claims = Jwt::decode_token_claims(service_id, user_id, x_type, key_value, token)?;
        Ok((claims.exp, claims.x_csrf))
    }

    /// Unsafely decodes a token, checks if service ID matches `iss` claim.
    /// If matched, returns the `sub` claim, which may be a user ID.
    /// The user ID must then be used to read a key that can safely decode the token.
    pub fn decode_unsafe(token: &str, service_id: Uuid) -> DriverResult<(Uuid, JwtClaimsType)> {
        let claims: JwtClaims = dangerous_unsafe_decode(token)
            .map_err(DriverError::Jsonwebtoken)?
            .claims;

        let iss = Uuid::parse_str(&claims.iss).map_err(DriverError::UuidParse)?;
        if service_id != iss {
            return Err(DriverError::JwtServiceMismatch);
        }

        let sub = Uuid::parse_str(&claims.sub).map_err(DriverError::UuidParse)?;
        let x_type = JwtClaimsType::from_i64(claims.x_type)?;
        Ok((sub, x_type))
    }

    pub fn decode_unsafe_service_id(token: &str, service_id: Uuid) -> DriverResult<(Uuid, Uuid)> {
        let claims: JwtClaims = dangerous_unsafe_decode(token)
            .map_err(DriverError::Jsonwebtoken)?
            .claims;

        let iss = Uuid::parse_str(&claims.iss).map_err(DriverError::UuidParse)?;
        if service_id == iss {
            return Err(DriverError::JwtServiceMismatch);
        }

        let sub = Uuid::parse_str(&claims.sub).map_err(DriverError::UuidParse)?;
        Ok((sub, iss))
    }

    /// Build user token by encoding access and refresh tokens.
    pub fn encode_user_token(
        driver: &Postgres,
        service: &Service,
        user: User,
        key: &KeyWithValue,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> DriverResult<UserToken> {
        let csrf = driver.csrf_create(&CsrfCreate::generate(refresh_token_expires, service.id))?;
        let (access_token, access_token_expires) = Jwt::encode_token(
            service.id,
            user.id,
            JwtClaimsType::AccessToken,
            &key.value,
            access_token_expires,
        )?;
        let (refresh_token, refresh_token_expires) = Jwt::encode_token_csrf(
            service.id,
            user.id,
            JwtClaimsType::RefreshToken,
            &csrf.key,
            &key.value,
            refresh_token_expires,
        )?;
        Ok(UserToken {
            user,
            access_token,
            access_token_expires,
            refresh_token,
            refresh_token_expires,
        })
    }

    pub fn encode_register_token(
        driver: &Postgres,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_expires: i64,
    ) -> DriverResult<String> {
        let csrf = driver.csrf_create(&CsrfCreate::generate(token_expires, service.id))?;
        let (token, _) = Jwt::encode_token_csrf(
            service.id,
            user.id,
            JwtClaimsType::RegisterToken,
            &csrf.key,
            &key.value,
            token_expires,
        )?;
        Ok(token)
    }

    pub fn decode_register_token(
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: &str,
    ) -> DriverResult<String> {
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::RegisterToken,
            &key.value,
            &token,
        );
        match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| DriverError::CsrfNotFoundOrUsed),
            Err(_err) => Err(DriverError::JwtInvalidOrExpired),
        }
    }

    pub fn encode_reset_password_token(
        driver: &Postgres,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_expires: i64,
    ) -> DriverResult<String> {
        let csrf = driver.csrf_create(&CsrfCreate::generate(token_expires, service.id))?;
        let (token, _) = Jwt::encode_token_csrf(
            service.id,
            user.id,
            JwtClaimsType::ResetPasswordToken,
            &csrf.key,
            &key.value,
            token_expires,
        )?;
        Ok(token)
    }

    pub fn decode_reset_password_token(
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: &str,
    ) -> DriverResult<String> {
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::ResetPasswordToken,
            &key.value,
            &token,
        );
        match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| DriverError::CsrfNotFoundOrUsed),
            Err(_err) => Err(DriverError::JwtInvalidOrExpired),
        }
    }

    pub fn encode_revoke_token(
        driver: &Postgres,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_expires: i64,
    ) -> DriverResult<String> {
        let csrf = driver.csrf_create(&CsrfCreate::generate(token_expires, service.id))?;
        let (revoke_token, _) = Jwt::encode_token_csrf(
            service.id,
            user.id,
            JwtClaimsType::RevokeToken,
            &csrf.key,
            &key.value,
            token_expires,
        )?;
        Ok(revoke_token)
    }

    pub fn decode_revoke_token(
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: &str,
    ) -> DriverResult<String> {
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::RevokeToken,
            &key.value,
            &token,
        );
        match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| DriverError::CsrfNotFoundOrUsed),
            Err(_err) => Err(DriverError::JwtInvalidOrExpired),
        }
    }

    pub fn decode_access_token(
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: &str,
    ) -> DriverResult<i64> {
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::AccessToken,
            &key.value,
            &token,
        );
        match decoded {
            Ok((access_token_expires, _)) => Ok(access_token_expires),
            Err(_err) => Err(DriverError::JwtInvalidOrExpired),
        }
    }

    pub fn decode_refresh_token(
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: &str,
    ) -> DriverResult<String> {
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::RefreshToken,
            &key.value,
            &token,
        );
        match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| DriverError::CsrfNotFoundOrUsed),
            Err(_err) => Err(DriverError::JwtInvalidOrExpired),
        }
    }

    pub fn decode_csrf_key(
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_type: JwtClaimsType,
        token: &str,
    ) -> DriverResult<Option<String>> {
        match Jwt::decode_token(service.id, user.id, token_type, &key.value, &token) {
            Ok((_, csrf_key)) => Ok(csrf_key),
            Err(_err) => Err(DriverError::JwtInvalidOrExpired),
        }
    }

    fn decode_token_claims(
        service_id: Uuid,
        user_id: Uuid,
        x_type: JwtClaimsType,
        key_value: &str,
        token: &str,
    ) -> DriverResult<JwtClaims> {
        let validation = JwtClaims::validation(service_id.to_string(), user_id.to_string());
        let data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(key_value.as_bytes()),
            &validation,
        )
        .map_err(DriverError::Jsonwebtoken)?;
        if data.claims.x_type != x_type.to_i64() {
            return Err(DriverError::JwtClaimsTypeMismatch);
        }
        Ok(data.claims)
    }
}
