use crate::prelude::*;
use diesel::PgConnection;
use jsonwebtoken::{dangerous_unsafe_decode, DecodingKey, EncodingKey, Header, Validation};

/// JSON web token types.
#[derive(Debug)]
pub enum JwtType {
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

impl JwtType {
    /// Returns i64 representation of type.
    pub fn to_i64(&self) -> i64 {
        match self {
            JwtType::AccessToken => 0,
            JwtType::RefreshToken => 1,
            JwtType::RegisterToken => 2,
            JwtType::ResetPasswordToken => 3,
            JwtType::RevokeToken => 4,
        }
    }

    /// Returns type or error if not valid.
    pub fn from_i64(value: i64) -> DriverResult<Self> {
        match value {
            0 => Ok(JwtType::AccessToken),
            1 => Ok(JwtType::RefreshToken),
            2 => Ok(JwtType::RegisterToken),
            3 => Ok(JwtType::ResetPasswordToken),
            4 => Ok(JwtType::RevokeToken),
            _ => Err(DriverError::JwtTypeInvalid),
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
    /// Returns new token of type without CSRF code.
    fn new<IS, SU>(iss: IS, sub: SU, exp: Duration, x_type: JwtType) -> Self
    where
        IS: Into<String>,
        SU: Into<String>,
    {
        let dt = Utc::now() + exp;
        JwtClaims {
            iss: iss.into(),
            sub: sub.into(),
            exp: dt.timestamp(),
            x_type: x_type.to_i64(),
            x_csrf: None,
        }
    }

    /// Returns new token of type with CSRF code.
    fn new_csrf<IS, SU, CS>(iss: IS, sub: SU, exp: Duration, x_type: JwtType, x_csrf: CS) -> Self
    where
        IS: Into<String>,
        SU: Into<String>,
        CS: Into<String>,
    {
        let mut claims = JwtClaims::new(iss, sub, exp, x_type);
        claims.x_csrf = Some(x_csrf.into());
        claims
    }

    /// Returns validation rules for decoding a token with issuer and subject.
    fn validation<IS, SU>(iss: IS, sub: SU) -> Validation
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

/// JSON web tokens.
#[derive(Debug)]
pub struct Jwt;

impl Jwt {
    /// Unsafely decodes a token, checks if service ID matches `iss` claim.
    /// If matched, returns the `sub` claim, which may be a user ID and the token type.
    /// The user ID must be used to read a key that can safely decode the token.
    pub fn decode_unsafe_user(token: &str, service_id: Uuid) -> DriverResult<(Uuid, JwtType)> {
        let claims: JwtClaims = dangerous_unsafe_decode(token)
            .map_err(DriverError::Jsonwebtoken)?
            .claims;

        let iss = Uuid::parse_str(&claims.iss).map_err(DriverError::UuidParse)?;
        if service_id != iss {
            return Err(DriverError::JwtServiceMismatch);
        }

        let sub = Uuid::parse_str(&claims.sub).map_err(DriverError::UuidParse)?;
        let x_type = JwtType::from_i64(claims.x_type)?;
        Ok((sub, x_type))
    }

    /// Encode and return access and refresh tokens for a user with key.
    pub fn encode_user(
        conn: &PgConnection,
        service: &Service,
        user: User,
        key: &KeyWithValue,
        access_token_expires: Duration,
        refresh_token_expires: Duration,
    ) -> DriverResult<UserToken> {
        let (access_token, access_token_expires) = Self::encode(
            service.id,
            user.id,
            JwtType::AccessToken,
            &key.value,
            access_token_expires,
        )?;
        let (refresh_token, refresh_token_expires) = Self::encode_csrf(
            conn,
            service.id,
            user.id,
            JwtType::RefreshToken,
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

    /// Safely decode access token for user with key.
    /// Returns expiry time.
    pub fn decode_access<T: AsRef<str>>(
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: T,
    ) -> DriverResult<i64> {
        let (exp, _) = Self::decode(
            service.id,
            user.id,
            JwtType::AccessToken,
            &key.value,
            token.as_ref(),
        )?;
        Ok(exp)
    }

    /// Safely decode refresh token for user with key and verify CSRF key.
    pub fn decode_refresh<T: AsRef<str>>(
        conn: &PgConnection,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: T,
    ) -> DriverResult<()> {
        let (_, csrf_key) = Self::decode(
            service.id,
            user.id,
            JwtType::RefreshToken,
            &key.value,
            token.as_ref(),
        )?;
        Csrf::verify(conn, service.id, csrf_key)?;
        Ok(())
    }

    /// Encode and return register token for user with key.
    pub fn encode_register(
        conn: &PgConnection,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_expires: Duration,
    ) -> DriverResult<String> {
        let (token, _) = Self::encode_csrf(
            conn,
            service.id,
            user.id,
            JwtType::RegisterToken,
            &key.value,
            token_expires,
        )?;
        Ok(token)
    }

    /// Safely decode register token for user with key and verify CSRF key.
    pub fn decode_register<T: AsRef<str>>(
        conn: &PgConnection,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: T,
    ) -> DriverResult<()> {
        let (_, csrf_key) = Self::decode(
            service.id,
            user.id,
            JwtType::RegisterToken,
            &key.value,
            token.as_ref(),
        )?;
        Csrf::verify(conn, service.id, csrf_key)?;
        Ok(())
    }

    /// Encode and return reset password token for user with key.
    pub fn encode_reset_password(
        conn: &PgConnection,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_expires: Duration,
    ) -> DriverResult<String> {
        let (token, _) = Self::encode_csrf(
            conn,
            service.id,
            user.id,
            JwtType::ResetPasswordToken,
            &key.value,
            token_expires,
        )?;
        Ok(token)
    }

    /// Safely decode reset password token for user with key and verify CSRF key.
    pub fn decode_reset_password<T: AsRef<str>>(
        conn: &PgConnection,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: T,
    ) -> DriverResult<()> {
        let (_, csrf_key) = Self::decode(
            service.id,
            user.id,
            JwtType::ResetPasswordToken,
            &key.value,
            token.as_ref(),
        )?;
        Csrf::verify(conn, service.id, csrf_key)?;
        Ok(())
    }

    /// Encode and return revoke token for user with key.
    pub fn encode_revoke(
        conn: &PgConnection,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_expires: Duration,
    ) -> DriverResult<String> {
        let (token, _) = Self::encode_csrf(
            conn,
            service.id,
            user.id,
            JwtType::RevokeToken,
            &key.value,
            token_expires,
        )?;
        Ok(token)
    }

    /// Safely decode revoke token for user with key and verify CSRF key.
    pub fn decode_revoke<T: AsRef<str>>(
        conn: &PgConnection,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: T,
    ) -> DriverResult<()> {
        let (_, csrf_key) = Self::decode(
            service.id,
            user.id,
            JwtType::RevokeToken,
            &key.value,
            token.as_ref(),
        )?;
        Csrf::verify(conn, service.id, csrf_key)?;
        Ok(())
    }

    /// Safely decode token of type for user with key, read CSRF to prevent verification.
    pub fn decode_csrf<T: AsRef<str>>(
        conn: &PgConnection,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_type: JwtType,
        token: T,
    ) -> DriverResult<()> {
        let (_, csrf_key) =
            Self::decode(service.id, user.id, token_type, &key.value, token.as_ref())?;
        if let Some(csrf_key) = csrf_key {
            Csrf::read(conn, &csrf_key)?;
        }
        Ok(())
    }

    /// Encode a token with key of type without a CSRF code, returns token and expiry time.
    fn encode(
        service_id: Uuid,
        user_id: Uuid,
        x_type: JwtType,
        key_value: &str,
        exp: Duration,
    ) -> DriverResult<(String, i64)> {
        let claims = JwtClaims::new(service_id.to_string(), user_id.to_string(), exp, x_type);
        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(key_value.as_bytes()),
        )
        .map_err(DriverError::Jsonwebtoken)?;
        Ok((token, claims.exp))
    }

    /// Encode a token with key of type with a CSRF code, returns token and expiry time.
    fn encode_csrf(
        conn: &PgConnection,
        service_id: Uuid,
        user_id: Uuid,
        x_type: JwtType,
        key_value: &str,
        exp: Duration,
    ) -> DriverResult<(String, i64)> {
        let csrf = Csrf::create(conn, &CsrfCreate::generate(exp, service_id))?;
        let claims = JwtClaims::new_csrf(
            service_id.to_string(),
            user_id.to_string(),
            exp,
            x_type,
            csrf.value(),
        );
        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(key_value.as_bytes()),
        )
        .map_err(DriverError::Jsonwebtoken)?;
        Ok((token, claims.exp))
    }

    /// Safely decodes a token with key, returns expiry time and optional CSRF key.
    /// This will return an error if the subject or issuer claims do not match the server
    /// and user ID, if the token is expired, or if the type is unexpected.
    fn decode(
        service_id: Uuid,
        user_id: Uuid,
        x_type: JwtType,
        key_value: &str,
        token: &str,
    ) -> DriverResult<(i64, Option<String>)> {
        let validation = JwtClaims::validation(service_id.to_string(), user_id.to_string());
        let data = jsonwebtoken::decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(key_value.as_bytes()),
            &validation,
        )
        .map_err(DriverError::Jsonwebtoken)?;
        if data.claims.x_type != x_type.to_i64() {
            return Err(DriverError::JwtTypeMismatch);
        }
        Ok((data.claims.exp, data.claims.x_csrf))
    }
}
