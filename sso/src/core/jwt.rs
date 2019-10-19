use crate::{CoreError, CoreResult};
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

    pub fn from_i64(value: i64) -> CoreResult<Self> {
        match value {
            0 => Ok(JwtClaimsType::AccessToken),
            1 => Ok(JwtClaimsType::RefreshToken),
            2 => Ok(JwtClaimsType::ResetPasswordToken),
            3 => Ok(JwtClaimsType::UpdateEmailRevokeToken),
            4 => Ok(JwtClaimsType::UpdatePasswordRevokeToken),
            _ => Err(CoreError::JwtClaimsTypeInvalid),
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
    ) -> CoreResult<(String, i64)> {
        let claims = JwtClaims::new(service_id.to_string(), user_id.to_string(), exp, x_type);
        let token = encode(&Header::default(), &claims, key_value.as_bytes())
            .map_err(CoreError::Jsonwebtoken)?;
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
    ) -> CoreResult<(String, i64)> {
        let claims = JwtClaims::new_csrf(
            service_id.to_string(),
            user_id.to_string(),
            exp,
            x_type,
            x_csrf,
        );
        let token = encode(&Header::default(), &claims, key_value.as_bytes())
            .map_err(CoreError::Jsonwebtoken)?;
        Ok((token, claims.exp))
    }

    /// Safely decodes a token, returns expiry time and optional CSRF key.
    pub fn decode_token(
        service_id: Uuid,
        user_id: Uuid,
        x_type: JwtClaimsType,
        key_value: &str,
        token: &str,
    ) -> CoreResult<(i64, Option<String>)> {
        let claims = Jwt::decode_token_claims(service_id, user_id, x_type, key_value, token)?;
        Ok((claims.exp, claims.x_csrf))
    }

    /// Unsafely decodes a token, checks if service ID matches `iss` claim.
    /// If matched, returns the `sub` claim, which may be a user ID.
    /// The user ID must then be used to read a key that can safely decode the token.
    pub fn decode_unsafe(token: &str, service_id: Uuid) -> CoreResult<(Uuid, JwtClaimsType)> {
        let claims: JwtClaims = dangerous_unsafe_decode(token)
            .map_err(CoreError::Jsonwebtoken)?
            .claims;

        let iss = Uuid::parse_str(&claims.iss).map_err(CoreError::UuidParse)?;
        if service_id != iss {
            return Err(CoreError::JwtServiceMismatch);
        }

        let sub = Uuid::parse_str(&claims.sub).map_err(CoreError::UuidParse)?;
        let x_type = JwtClaimsType::from_i64(claims.x_type)?;
        Ok((sub, x_type))
    }

    fn decode_token_claims(
        service_id: Uuid,
        user_id: Uuid,
        x_type: JwtClaimsType,
        key_value: &str,
        token: &str,
    ) -> CoreResult<JwtClaims> {
        let validation = JwtClaims::validation(service_id.to_string(), user_id.to_string());
        let data = decode::<JwtClaims>(token, key_value.as_bytes(), &validation)
            .map_err(CoreError::Jsonwebtoken)?;
        if data.claims.x_type != x_type.to_i64() {
            return Err(CoreError::JwtClaimsTypeMismatch);
        }
        Ok(data.claims)
    }
}
