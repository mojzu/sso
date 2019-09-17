use crate::CoreError;
use serde::{Serialize, Serializer};
use std::str::FromStr;

/// Audit type authenticate pub error.
const AUDIT_TYPE_AUTHENTICATE_ERROR: &str = "ark_auth:error:authenticate";
/// Audit type login.
pub const AUDIT_TYPE_LOGIN: &str = "ark_auth:login";
/// Audit type login_error.
pub const AUDIT_TYPE_LOGIN_ERROR: &str = "ark_auth:error:login";
/// Audit type reset password.
pub const AUDIT_TYPE_RESET_PASSWORD: &str = "ark_auth:reset_password";
/// Audit type reset password error.
pub const AUDIT_TYPE_RESET_PASSWORD_ERROR: &str = "ark_auth:error:reset_password";
/// Audit type reset password confirm.
pub const AUDIT_TYPE_RESET_PASSWORD_CONFIRM: &str = "ark_auth:reset_password_confirm";
/// Audit type reset password confirm error.
pub const AUDIT_TYPE_RESET_PASSWORD_CONFIRM_ERROR: &str = "ark_auth:error:reset_password_confirm";
/// Audit type update email.
pub const AUDIT_TYPE_UPDATE_EMAIL: &str = "ark_auth:update_email";
/// Audit type update email error.
pub const AUDIT_TYPE_UPDATE_EMAIL_ERROR: &str = "ark_auth:error:update_email";
/// Audit type update email revoke.
pub const AUDIT_TYPE_UPDATE_EMAIL_REVOKE: &str = "ark_auth:update_email_revoke";
/// Audit type update email revoke error.
pub const AUDIT_TYPE_UPDATE_EMAIL_REVOKE_ERROR: &str = "ark_auth:error:update_email_revoke";
/// Audit type update password.
pub const AUDIT_TYPE_UPDATE_PASSWORD: &str = "ark_auth:update_password";
/// Audit type update password error.
pub const AUDIT_TYPE_UPDATE_PASSWORD_ERROR: &str = "ark_auth:error:update_password";
/// Audit type update password revoke.
pub const AUDIT_TYPE_UPDATE_PASSWORD_REVOKE: &str = "ark_auth:update_password_revoke";
/// Audit type update password revoke error.
pub const AUDIT_TYPE_UPDATE_PASSWORD_REVOKE_ERROR: &str = "ark_auth:error:update_password_revoke";
/// Audit type OAuth2 login.
pub const AUDIT_TYPE_OAUTH2_LOGIN: &str = "ark_auth:oauth2_login";
/// Audit type OAuth2 login error.
pub const AUDIT_TYPE_OAUTH2_LOGIN_ERROR: &str = "ark_auth:error:oauth2_login";
/// Audit type key verify error.
pub const AUDIT_TYPE_KEY_VERIFY_ERROR: &str = "ark_auth:error:key_verify";
/// Audit type key revoke.
pub const AUDIT_TYPE_KEY_REVOKE: &str = "ark_auth:key_revoke";
/// Audit type key revoke error.
pub const AUDIT_TYPE_KEY_REVOKE_ERROR: &str = "ark_auth:error:key_revoke";
/// Audit type token verify error.
pub const AUDIT_TYPE_TOKEN_VERIFY_ERROR: &str = "ark_auth:error:token_verify";
/// Audit type token refresh.
pub const AUDIT_TYPE_TOKEN_REFRESH: &str = "ark_auth:token_refresh";
/// Audit type token refresh error.
pub const AUDIT_TYPE_TOKEN_REFRESH_ERROR: &str = "ark_auth:error:token_refresh";
/// Audit type token revoke.
pub const AUDIT_TYPE_TOKEN_REVOKE: &str = "ark_auth:token_revoke";
/// Audit type token revoke error.
pub const AUDIT_TYPE_TOKEN_REVOKE_ERROR: &str = "ark_auth:error:token_revoke";
/// Audit type TOTP error.
pub const AUDIT_TYPE_TOTP_ERROR: &str = "ark_auth:error:totp";

/// Audit types.
#[derive(Debug, Copy, Clone)]
pub enum AuditType {
    AuthenticateError,
    Login,
    LoginError,
    ResetPassword,
    ResetPasswordError,
    ResetPasswordConfirm,
    ResetPasswordConfirmError,
    UpdateEmail,
    UpdateEmailError,
    UpdateEmailRevoke,
    UpdateEmailRevokeError,
    UpdatePassword,
    UpdatePasswordError,
    UpdatePasswordRevoke,
    UpdatePasswordRevokeError,
    Oauth2Login,
    Oauth2LoginError,
    KeyVerifyError,
    KeyRevoke,
    KeyRevokeError,
    TokenVerifyError,
    TokenRefresh,
    TokenRefreshError,
    TokenRevoke,
    TokenRevokeError,
    TotpError,
}

impl AuditType {
    /// Return string reference.
    pub fn as_str(&self) -> &str {
        match self {
            Self::AuthenticateError => AUDIT_TYPE_AUTHENTICATE_ERROR,
            Self::Login => AUDIT_TYPE_LOGIN,
            Self::LoginError => AUDIT_TYPE_LOGIN_ERROR,
            Self::ResetPassword => AUDIT_TYPE_RESET_PASSWORD,
            Self::ResetPasswordError => AUDIT_TYPE_RESET_PASSWORD_ERROR,
            Self::ResetPasswordConfirm => AUDIT_TYPE_RESET_PASSWORD_CONFIRM,
            Self::ResetPasswordConfirmError => AUDIT_TYPE_RESET_PASSWORD_CONFIRM_ERROR,
            Self::UpdateEmail => AUDIT_TYPE_UPDATE_EMAIL,
            Self::UpdateEmailError => AUDIT_TYPE_UPDATE_EMAIL_ERROR,
            Self::UpdateEmailRevoke => AUDIT_TYPE_UPDATE_EMAIL_REVOKE,
            Self::UpdateEmailRevokeError => AUDIT_TYPE_UPDATE_EMAIL_REVOKE_ERROR,
            Self::UpdatePassword => AUDIT_TYPE_UPDATE_PASSWORD,
            Self::UpdatePasswordError => AUDIT_TYPE_UPDATE_PASSWORD_ERROR,
            Self::UpdatePasswordRevoke => AUDIT_TYPE_UPDATE_PASSWORD_REVOKE,
            Self::UpdatePasswordRevokeError => AUDIT_TYPE_UPDATE_PASSWORD_REVOKE_ERROR,
            Self::Oauth2Login => AUDIT_TYPE_OAUTH2_LOGIN,
            Self::Oauth2LoginError => AUDIT_TYPE_OAUTH2_LOGIN_ERROR,
            Self::KeyVerifyError => AUDIT_TYPE_KEY_VERIFY_ERROR,
            Self::KeyRevoke => AUDIT_TYPE_KEY_REVOKE,
            Self::KeyRevokeError => AUDIT_TYPE_KEY_REVOKE_ERROR,
            Self::TokenVerifyError => AUDIT_TYPE_TOKEN_VERIFY_ERROR,
            Self::TokenRefresh => AUDIT_TYPE_TOKEN_REFRESH,
            Self::TokenRefreshError => AUDIT_TYPE_TOKEN_REFRESH_ERROR,
            Self::TokenRevoke => AUDIT_TYPE_TOKEN_REVOKE,
            Self::TokenRevokeError => AUDIT_TYPE_TOKEN_REVOKE_ERROR,
            Self::TotpError => AUDIT_TYPE_TOTP_ERROR,
        }
    }
}

impl FromStr for AuditType {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            AUDIT_TYPE_AUTHENTICATE_ERROR => Ok(Self::AuthenticateError),
            AUDIT_TYPE_LOGIN => Ok(Self::Login),
            AUDIT_TYPE_LOGIN_ERROR => Ok(Self::LoginError),
            AUDIT_TYPE_RESET_PASSWORD => Ok(Self::ResetPassword),
            AUDIT_TYPE_RESET_PASSWORD_ERROR => Ok(Self::ResetPasswordError),
            AUDIT_TYPE_RESET_PASSWORD_CONFIRM => Ok(Self::ResetPasswordConfirm),
            AUDIT_TYPE_RESET_PASSWORD_CONFIRM_ERROR => Ok(Self::ResetPasswordConfirmError),
            AUDIT_TYPE_UPDATE_EMAIL => Ok(Self::UpdateEmail),
            AUDIT_TYPE_UPDATE_EMAIL_ERROR => Ok(Self::UpdateEmailError),
            AUDIT_TYPE_UPDATE_EMAIL_REVOKE => Ok(Self::UpdateEmailRevoke),
            AUDIT_TYPE_UPDATE_EMAIL_REVOKE_ERROR => Ok(Self::UpdateEmailRevokeError),
            AUDIT_TYPE_UPDATE_PASSWORD => Ok(Self::UpdatePassword),
            AUDIT_TYPE_UPDATE_PASSWORD_ERROR => Ok(Self::UpdatePasswordError),
            AUDIT_TYPE_UPDATE_PASSWORD_REVOKE => Ok(Self::UpdatePasswordRevoke),
            AUDIT_TYPE_UPDATE_PASSWORD_REVOKE_ERROR => Ok(Self::UpdatePasswordRevokeError),
            AUDIT_TYPE_OAUTH2_LOGIN => Ok(Self::Oauth2Login),
            AUDIT_TYPE_OAUTH2_LOGIN_ERROR => Ok(Self::Oauth2LoginError),
            AUDIT_TYPE_KEY_VERIFY_ERROR => Ok(Self::KeyVerifyError),
            AUDIT_TYPE_KEY_REVOKE => Ok(Self::KeyRevoke),
            AUDIT_TYPE_KEY_REVOKE_ERROR => Ok(Self::KeyRevokeError),
            AUDIT_TYPE_TOKEN_VERIFY_ERROR => Ok(Self::TokenVerifyError),
            AUDIT_TYPE_TOKEN_REFRESH => Ok(Self::TokenRefresh),
            AUDIT_TYPE_TOKEN_REFRESH_ERROR => Ok(Self::TokenRefreshError),
            AUDIT_TYPE_TOKEN_REVOKE => Ok(Self::TokenRevoke),
            AUDIT_TYPE_TOKEN_REVOKE_ERROR => Ok(Self::TokenRevokeError),
            AUDIT_TYPE_TOTP_ERROR => Ok(Self::TotpError),
            _ => Err(CoreError::AuditType),
        }
    }
}

impl Serialize for AuditType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}
