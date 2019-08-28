use crate::core::Error;
use jsonwebtoken::{dangerous_unsafe_decode, decode, encode, Header, Validation};
use uuid::Uuid;

#[derive(Debug)]
pub enum ClaimsType {
    AccessToken,
    RefreshToken,
    ResetPasswordToken,
    UpdateEmailRevokeToken,
    UpdatePasswordRevokeToken,
}

impl ClaimsType {
    pub fn to_i64(&self) -> i64 {
        match self {
            ClaimsType::AccessToken => 0,
            ClaimsType::RefreshToken => 1,
            ClaimsType::ResetPasswordToken => 2,
            ClaimsType::UpdateEmailRevokeToken => 3,
            ClaimsType::UpdatePasswordRevokeToken => 4,
        }
    }

    pub fn from_i64(value: i64) -> Result<Self, Error> {
        match value {
            0 => Ok(ClaimsType::AccessToken),
            1 => Ok(ClaimsType::RefreshToken),
            2 => Ok(ClaimsType::ResetPasswordToken),
            3 => Ok(ClaimsType::UpdateEmailRevokeToken),
            4 => Ok(ClaimsType::UpdatePasswordRevokeToken),
            _ => Err(Error::BadRequest),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    sub: String,
    exp: i64,
    x_type: i64,
    x_csrf: Option<String>,
}

impl Claims {
    pub fn new<T1, T2>(iss: T1, sub: T2, exp: i64, x_type: ClaimsType) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        let dt = chrono::Utc::now();
        let exp = dt.timestamp() + exp;
        Claims {
            iss: iss.into(),
            sub: sub.into(),
            exp,
            x_type: x_type.to_i64(),
            x_csrf: None,
        }
    }

    pub fn new_csrf<T1, T2, T3>(iss: T1, sub: T2, exp: i64, x_type: ClaimsType, x_csrf: T3) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
        T3: Into<String>,
    {
        let mut claims = Claims::new(iss, sub, exp, x_type);
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

/// Encode a token, returns token and expiry time.
pub fn encode_token(
    service_id: Uuid,
    user_id: Uuid,
    x_type: ClaimsType,
    key_value: &str,
    exp: i64,
) -> Result<(String, i64), Error> {
    let claims = Claims::new(service_id.to_string(), user_id.to_string(), exp, x_type);
    let token =
        encode(&Header::default(), &claims, key_value.as_bytes()).map_err(Error::Jsonwebtoken)?;
    Ok((token, claims.exp))
}

/// Encode a CSRF token, returns token and expiry time.
pub fn encode_token_csrf(
    service_id: Uuid,
    user_id: Uuid,
    x_type: ClaimsType,
    x_csrf: &str,
    key_value: &str,
    exp: i64,
) -> Result<(String, i64), Error> {
    let claims = Claims::new_csrf(
        service_id.to_string(),
        user_id.to_string(),
        exp,
        x_type,
        x_csrf,
    );
    let token =
        encode(&Header::default(), &claims, key_value.as_bytes()).map_err(Error::Jsonwebtoken)?;
    Ok((token, claims.exp))
}

/// Safely decodes a token, returns expiry time and optional CSRF key.
pub fn decode_token(
    service_id: Uuid,
    user_id: Uuid,
    x_type: ClaimsType,
    key_value: &str,
    token: &str,
) -> Result<(i64, Option<String>), Error> {
    let claims = decode_token_claims(service_id, user_id, x_type, key_value, token)?;
    Ok((claims.exp, claims.x_csrf))
}

/// Unsafely decodes a token, checks if service ID matches `iss` claim.
/// If matched, returns the `sub` claim, which may be a user ID.
/// The user ID must then be used to read a key that can safely decode the token.
pub fn decode_unsafe(token: &str, service_id: Uuid) -> Result<(Uuid, ClaimsType), Error> {
    let claims: Claims = dangerous_unsafe_decode(token)
        .map_err(Error::Jsonwebtoken)?
        .claims;

    let iss = Uuid::parse_str(&claims.iss).map_err(Error::UuidParse)?;
    if service_id != iss {
        return Err(Error::BadRequest);
    }

    let sub = Uuid::parse_str(&claims.sub).map_err(Error::UuidParse)?;
    let x_type = ClaimsType::from_i64(claims.x_type)?;
    Ok((sub, x_type))
}

fn decode_token_claims(
    service_id: Uuid,
    user_id: Uuid,
    x_type: ClaimsType,
    key_value: &str,
    token: &str,
) -> Result<Claims, Error> {
    let validation = Claims::validation(service_id.to_string(), user_id.to_string());
    let data =
        decode::<Claims>(token, key_value.as_bytes(), &validation).map_err(Error::Jsonwebtoken)?;
    if data.claims.x_type != x_type.to_i64() {
        return Err(Error::BadRequest);
    }
    Ok(data.claims)
}
