use crate::core::Error;
use jsonwebtoken::{dangerous_unsafe_decode, decode, encode, Header, Validation};

// TODO(refactor): Improve handling of decode errors.
// ServerError::Core CoreError::Jsonwebtoken invalid signature

#[derive(Debug)]
pub enum ClaimsType {
    AccessToken = 0,
    RefreshToken = 1,
    ResetPasswordToken = 2,
    UpdateEmailRevokeToken = 3,
    UpdatePasswordRevokeToken = 4,
}

impl ClaimsType {
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
    pub fn new(iss: &str, sub: &str, exp: i64, x_type: ClaimsType, x_csrf: Option<&str>) -> Self {
        let dt = chrono::Utc::now();
        let exp = dt.timestamp() + exp;
        Claims {
            iss: iss.to_owned(),
            sub: sub.to_owned(),
            exp,
            x_type: x_type as i64,
            x_csrf: x_csrf.map(|x| x.to_owned()),
        }
    }

    pub fn validation(iss: &str, sub: &str) -> Validation {
        Validation {
            iss: Some(iss.to_owned()),
            sub: Some(sub.to_owned()),
            ..Validation::default()
        }
    }
}

/// Encode a token, returns token and expiry time.
pub fn encode_token(
    service_id: &str,
    user_id: &str,
    x_type: ClaimsType,
    x_csrf: Option<&str>,
    key_value: &str,
    exp: i64,
) -> Result<(String, i64), Error> {
    let claims = Claims::new(service_id, user_id, exp, x_type, x_csrf);
    let token =
        encode(&Header::default(), &claims, key_value.as_bytes()).map_err(Error::Jsonwebtoken)?;
    Ok((token, claims.exp))
}

/// Safely decodes a token, returns token and expiry time.
pub fn decode_token(
    service_id: &str,
    user_id: &str,
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
pub fn decode_unsafe(token: &str, service_id: &str) -> Result<(String, ClaimsType), Error> {
    let claims: Claims = dangerous_unsafe_decode(token)
        .map_err(Error::Jsonwebtoken)?
        .claims;
    if service_id != claims.iss {
        return Err(Error::BadRequest);
    }
    let x_type = ClaimsType::from_i64(claims.x_type)?;
    Ok((claims.sub, x_type))
}

fn decode_token_claims(
    service_id: &str,
    user_id: &str,
    x_type: ClaimsType,
    key_value: &str,
    token: &str,
) -> Result<Claims, Error> {
    let validation = Claims::validation(service_id, user_id);
    let data =
        decode::<Claims>(token, key_value.as_bytes(), &validation).map_err(Error::Jsonwebtoken)?;
    if data.claims.x_type != x_type as i64 {
        return Err(Error::BadRequest);
    }
    Ok(data.claims)
}
