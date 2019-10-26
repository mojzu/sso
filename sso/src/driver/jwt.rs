use crate::{
    CsrfCreate, Driver, DriverError, DriverResult, KeyWithValue, Service, User, UserToken,
};
use jsonwebtoken::{dangerous_unsafe_decode, decode, encode, Header, Validation};
use uuid::Uuid;

/// JSON web token maximum length.
pub const JWT_MAX_LEN: usize = 1000;

/// JSON web token claims types.
#[derive(Debug)]
pub enum JwtClaimsType {
    AccessToken,
    RefreshToken,
    ResetPasswordToken,
    UpdateEmailRevokeToken,
    UpdatePasswordRevokeToken,
}

impl JwtClaimsType {
    pub fn to_i64(&self) -> i64 {
        match self {
            JwtClaimsType::AccessToken => 0,
            JwtClaimsType::RefreshToken => 1,
            JwtClaimsType::ResetPasswordToken => 2,
            JwtClaimsType::UpdateEmailRevokeToken => 3,
            JwtClaimsType::UpdatePasswordRevokeToken => 4,
        }
    }

    pub fn from_i64(value: i64) -> DriverResult<Self> {
        match value {
            0 => Ok(JwtClaimsType::AccessToken),
            1 => Ok(JwtClaimsType::RefreshToken),
            2 => Ok(JwtClaimsType::ResetPasswordToken),
            3 => Ok(JwtClaimsType::UpdateEmailRevokeToken),
            4 => Ok(JwtClaimsType::UpdatePasswordRevokeToken),
            _ => Err(DriverError::JwtClaimsTypeInvalid),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    iss: String,
    sub: String,
    exp: i64,
    x_type: i64,
    x_csrf: Option<String>,
}

impl JwtClaims {
    pub fn new<T1, T2>(iss: T1, sub: T2, exp: i64, x_type: JwtClaimsType) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
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

    pub fn new_csrf<T1, T2, T3>(
        iss: T1,
        sub: T2,
        exp: i64,
        x_type: JwtClaimsType,
        x_csrf: T3,
    ) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
        T3: Into<String>,
    {
        let mut claims = JwtClaims::new(iss, sub, exp, x_type);
        claims.x_csrf = Some(x_csrf.into());
        claims
    }

    pub fn validation<T1, T2>(iss: T1, sub: T2) -> Validation
    where
        T1: Into<String>,
        T2: Into<String>,
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
        let token = encode(&Header::default(), &claims, key_value.as_bytes())
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
        let token = encode(&Header::default(), &claims, key_value.as_bytes())
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

    /// Build user token by encoding access and refresh tokens.
    pub fn encode_user_token(
        driver: &dyn Driver,
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
    pub fn encode_reset_password_token(
        driver: &dyn Driver,
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

    pub fn encode_update_email_token(
        driver: &dyn Driver,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_expires: i64,
    ) -> DriverResult<String> {
        let csrf = driver.csrf_create(&CsrfCreate::generate(token_expires, service.id))?;
        let (revoke_token, _) = Jwt::encode_token_csrf(
            service.id,
            user.id,
            JwtClaimsType::UpdateEmailRevokeToken,
            &csrf.key,
            &key.value,
            token_expires,
        )?;
        Ok(revoke_token)
    }

    pub fn decode_update_email_token(
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: &str,
    ) -> DriverResult<String> {
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::UpdateEmailRevokeToken,
            &key.value,
            &token,
        );
        match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| DriverError::CsrfNotFoundOrUsed),
            Err(_err) => Err(DriverError::JwtInvalidOrExpired),
        }
    }

    pub fn encode_update_password_token(
        driver: &dyn Driver,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_expires: i64,
    ) -> DriverResult<String> {
        let csrf = driver.csrf_create(&CsrfCreate::generate(token_expires, service.id))?;
        let (revoke_token, _) = Jwt::encode_token_csrf(
            service.id,
            user.id,
            JwtClaimsType::UpdatePasswordRevokeToken,
            &csrf.key,
            &key.value,
            token_expires,
        )?;
        Ok(revoke_token)
    }

    pub fn decode_update_password_token(
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: &str,
    ) -> DriverResult<String> {
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::UpdatePasswordRevokeToken,
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
        let data = decode::<JwtClaims>(token, key_value.as_bytes(), &validation)
            .map_err(DriverError::Jsonwebtoken)?;
        if data.claims.x_type != x_type.to_i64() {
            return Err(DriverError::JwtClaimsTypeMismatch);
        }
        Ok(data.claims)
    }
}