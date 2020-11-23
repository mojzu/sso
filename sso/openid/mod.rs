//! # OpenID
//!
//! [Specification](https://openid.net/developers/specs/)

// todo: OpenID Connect module, other refactoring?

/// Provider configuration response
/// [RFC](https://openid.net/specs/openid-connect-discovery-1_0.html#ProviderConfigurationResponse)
#[derive(Debug)]
pub struct ProviderConfigurationResponse {
    issuer: String,
}
