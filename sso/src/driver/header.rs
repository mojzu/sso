use http::{HeaderMap, HeaderValue};
use uuid::Uuid;

/// Authorisation header.
pub const HEADER_AUTHORISATION: &str = "Authorization";

/// User authorisation header.
pub const HEADER_USER_AUTHORISATION: &str = "User-Authorization";

/// Service authorisation header.
pub const HEADER_SERVICE_AUTHORISATION: &str = "Service-Authorization";

/// User-Agent header.
pub const HEADER_USER_AGENT: &str = "User-Agent";

/// X-Forwarded-For header.
pub const HEADER_X_FORWARDED_FOR: &str = "X-Forwarded-For";

/// Grpc-Metadata-Sso-Key-Id header.
pub const HEADER_GRPC_METADATA_SSO_KEY_ID: &str = "Grpc-Metadata-Sso-Key-Id";

/// Grpc-Metadata-Sso-Service-Id header.
pub const HEADER_GRPC_METADATA_SSO_SERVICE_ID: &str = "Grpc-Metadata-Sso-Service-Id";

/// Grpc-Metadata-Sso-User-Key-Id header.
pub const HEADER_GRPC_METADATA_SSO_USER_KEY_ID: &str = "Grpc-Metadata-Sso-User-Key-Id";

/// Grpc-Metadata-Sso-User-Id header.
pub const HEADER_GRPC_METADATA_SSO_USER_ID: &str = "Grpc-Metadata-Sso-User-Id";

/// Sso-Key-Id header.
pub const HEADER_SSO_KEY_ID: &str = "Sso-Key-Id";

/// Sso-Service-Id header.
pub const HEADER_SSO_SERVICE_ID: &str = "Sso-Service-Id";

/// Sso-User-Key-Id header.
pub const HEADER_SSO_USER_KEY_ID: &str = "Sso-User-Key-Id";

/// Sso-User-Id header.
pub const HEADER_SSO_USER_ID: &str = "Sso-User-Id";

/// Extract Authorization header string.
pub fn header_authorisation(map: &HeaderMap<HeaderValue>) -> Option<String> {
    if let Some(x) = map.get(HEADER_AUTHORISATION) {
        match x.to_str() {
            Ok(x) => HeaderAuth::parse_key(x),
            Err(_e) => None,
        }
    } else {
        None
    }
}

/// Extract User-Authorization header string.
pub fn header_user_authorisation(map: &HeaderMap<HeaderValue>) -> Option<HeaderAuthType> {
    if let Some(x) = map.get(HEADER_USER_AUTHORISATION) {
        match x.to_str() {
            Ok(x) => HeaderAuth::parse_type(x),
            Err(_e) => None,
        }
    } else {
        None
    }
}

/// Extract Service-Authorization header string.
pub fn header_service_authorisation(map: &HeaderMap<HeaderValue>) -> Option<String> {
    if let Some(x) = map.get(HEADER_SERVICE_AUTHORISATION) {
        match x.to_str() {
            Ok(x) => HeaderAuth::parse_key(x),
            Err(_e) => None,
        }
    } else {
        None
    }
}

/// Extract User-Agent header string.
pub fn header_user_agent(map: &HeaderMap<HeaderValue>) -> String {
    if let Some(x) = map.get(HEADER_USER_AGENT) {
        match x.to_str() {
            Ok(x) => x.to_owned(),
            Err(_e) => "".to_owned(),
        }
    } else {
        "".to_owned()
    }
}

/// Extract X-Forwarded-For header string.
pub fn header_x_forwarded_for(map: &HeaderMap<HeaderValue>) -> Option<String> {
    if let Some(x) = map.get(HEADER_X_FORWARDED_FOR) {
        match x.to_str() {
            Ok(x) => Some(x.to_owned()),
            Err(_e) => None,
        }
    } else {
        None
    }
}

pub fn header_sso_key_id(map: &HeaderMap<HeaderValue>) -> Option<Uuid> {
    if let Some(x) = map
        .get(HEADER_SSO_KEY_ID)
        .or(map.get(HEADER_GRPC_METADATA_SSO_KEY_ID))
    {
        match x.to_str() {
            Ok(x) => match Uuid::parse_str(x) {
                Ok(x) => Some(x),
                Err(_e) => None,
            },
            Err(_e) => None,
        }
    } else {
        None
    }
}

pub fn header_sso_service_id(map: &HeaderMap<HeaderValue>) -> Option<Uuid> {
    if let Some(x) = map
        .get(HEADER_SSO_SERVICE_ID)
        .or(map.get(HEADER_GRPC_METADATA_SSO_SERVICE_ID))
    {
        match x.to_str() {
            Ok(x) => match Uuid::parse_str(x) {
                Ok(x) => Some(x),
                Err(_e) => None,
            },
            Err(_e) => None,
        }
    } else {
        None
    }
}

pub fn header_sso_user_key_id(map: &HeaderMap<HeaderValue>) -> Option<Uuid> {
    if let Some(x) = map
        .get(HEADER_SSO_USER_KEY_ID)
        .or(map.get(HEADER_GRPC_METADATA_SSO_USER_KEY_ID))
    {
        match x.to_str() {
            Ok(x) => match Uuid::parse_str(x) {
                Ok(x) => Some(x),
                Err(_e) => None,
            },
            Err(_e) => None,
        }
    } else {
        None
    }
}

pub fn header_sso_user_id(map: &HeaderMap<HeaderValue>) -> Option<Uuid> {
    if let Some(x) = map
        .get(HEADER_SSO_USER_ID)
        .or(map.get(HEADER_GRPC_METADATA_SSO_USER_ID))
    {
        match x.to_str() {
            Ok(x) => match Uuid::parse_str(x) {
                Ok(x) => Some(x),
                Err(_e) => None,
            },
            Err(_e) => None,
        }
    } else {
        None
    }
}

/// Header Traefik authentication data.
#[derive(Debug, Clone)]
pub struct HeaderAuthTraefik {
    pub key_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub user_key_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

/// Header authentication type.
#[derive(Debug, Clone, PartialEq)]
pub enum HeaderAuthType {
    Key(String),
    Token(String),
}

/// Header authentication data.
#[derive(Debug, Clone)]
pub enum HeaderAuth {
    Traefik(HeaderAuthTraefik),
    Header(HeaderAuthType),
    None,
}

impl HeaderAuth {
    /// Parse header value, returns key value.
    /// Formats: `$KEY`, `key $KEY`, `Bearer $KEY`
    fn parse_key(value: &str) -> Option<String> {
        let value = value.to_owned();
        if value.starts_with("key ") || value.starts_with("Bearer ") {
            let parts: Vec<&str> = value.split_whitespace().collect();
            if parts.len() > 1 {
                let value = parts[1].trim().to_owned();
                Some(value)
            } else {
                None
            }
        } else {
            Some(value)
        }
    }

    /// Parse header value, extract key or token.
    /// Formats: `$KEY`, `key $KEY`, `token $TOKEN`
    pub fn parse_type(value: &str) -> Option<HeaderAuthType> {
        let mut type_value = value.split_whitespace();
        let type_ = match type_value.next() {
            Some(type_) => type_,
            None => return None,
        };

        Some(match type_value.next() {
            Some(value) => match type_ {
                "token" => HeaderAuthType::Token(value.to_owned()),
                "key" => HeaderAuthType::Key(value.to_owned()),
                _ => HeaderAuthType::Key(value.to_owned()),
            },
            None => HeaderAuthType::Key(type_.to_owned()),
        })
    }

    pub fn from_header_map(map: &HeaderMap<HeaderValue>, traefik_enabled: bool) -> Self {
        // TODO(sam,refactor): Other headers: x-forwarded-host, x-forwarded-uri, x-real-ip.
        if traefik_enabled {
            Self::Traefik(HeaderAuthTraefik {
                key_id: header_sso_key_id(map),
                service_id: header_sso_service_id(map),
                user_key_id: header_sso_user_key_id(map),
                user_id: header_sso_user_id(map),
            })
        } else {
            match header_authorisation(map) {
                Some(x) => match Self::parse_type(&x) {
                    Some(x) => Self::Header(x),
                    None => Self::None,
                },
                None => Self::None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_auth_parses_type_none() {
        let x = HeaderAuth::parse_type("abcdefg").unwrap();
        assert_eq!(x, HeaderAuthType::Key("abcdefg".to_owned()));
    }

    #[test]
    fn header_auth_parses_type_key() {
        let x = HeaderAuth::parse_type("key abcdefg").unwrap();
        assert_eq!(x, HeaderAuthType::Key("abcdefg".to_owned()));
    }

    #[test]
    fn header_auth_parses_type_token() {
        let x = HeaderAuth::parse_type("token abcdefg").unwrap();
        assert_eq!(x, HeaderAuthType::Token("abcdefg".to_owned()));
    }
}
