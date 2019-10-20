use crate::{
    notify_msg::{EmailResetPassword, EmailUpdateEmail, EmailUpdatePassword},
    AuditBuilder, AuditMeta, CoreError, CoreResult, Csrf, Driver, Jwt, JwtClaimsType, Key, KeyType,
    KeyWithValue, NotifyActor, Service, ServiceRead, User, UserRead, UserToken,
    USER_PASSWORD_HASH_VERSION, USER_PASSWORD_MAX_LEN, USER_PASSWORD_MIN_LEN,
};
use actix::Addr;
use libreauth::oath::TOTPBuilder;
use libreauth::pass::HashBuilder;
use uuid::Uuid;

/// Authentication functions.
#[derive(Debug)]
pub struct Auth;

// TODO(refactor): Move this logic, other core methods into api/driver?

impl Auth {
    pub fn notify_email_reset_password(
        notify: &Addr<NotifyActor>,
        service: Service,
        user: User,
        token: String,
        audit: AuditMeta,
    ) -> CoreResult<()> {
        notify
            .try_send(EmailResetPassword::new(service, user, token, audit))
            .map_err(|_err| CoreError::NotifySendError)
    }

    pub fn encode_reset_password_token(
        driver: &dyn Driver,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_expires: i64,
    ) -> CoreResult<String> {
        let csrf = Auth::csrf_create(driver, service, token_expires)?;
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
    ) -> CoreResult<String> {
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::ResetPasswordToken,
            &key.value,
            &token,
        );
        match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| CoreError::CsrfNotFoundOrUsed),
            Err(_err) => Err(CoreError::JwtInvalidOrExpired),
        }
    }

    pub fn encode_update_email_token(
        driver: &dyn Driver,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_expires: i64,
    ) -> CoreResult<String> {
        let csrf = Auth::csrf_create(driver, service, token_expires)?;
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
    ) -> CoreResult<String> {
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::UpdateEmailRevokeToken,
            &key.value,
            &token,
        );
        match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| CoreError::CsrfNotFoundOrUsed),
            Err(_err) => Err(CoreError::JwtInvalidOrExpired),
        }
    }

    pub fn notify_email_update_email(
        notify: &Addr<NotifyActor>,
        service: Service,
        user: User,
        old_email: String,
        revoke_token: String,
        audit: AuditMeta,
    ) -> CoreResult<()> {
        notify
            .try_send(EmailUpdateEmail::new(
                service,
                user,
                old_email,
                revoke_token,
                audit,
            ))
            .map_err(|_err| CoreError::NotifySendError)
    }

    pub fn encode_update_password_token(
        driver: &dyn Driver,
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_expires: i64,
    ) -> CoreResult<String> {
        let csrf = Auth::csrf_create(driver, service, token_expires)?;
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

    pub fn notify_email_update_password(
        notify: &Addr<NotifyActor>,
        service: Service,
        user: User,
        revoke_token: String,
        audit: AuditMeta,
    ) -> CoreResult<()> {
        notify
            .try_send(EmailUpdatePassword::new(service, user, revoke_token, audit))
            .map_err(|_err| CoreError::NotifySendError)
    }

    pub fn decode_update_password_token(
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: &str,
    ) -> CoreResult<String> {
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::UpdatePasswordRevokeToken,
            &key.value,
            &token,
        );
        match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| CoreError::CsrfNotFoundOrUsed),
            Err(_err) => Err(CoreError::JwtInvalidOrExpired),
        }
    }

    pub fn decode_access_token(
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: &str,
    ) -> CoreResult<i64> {
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::AccessToken,
            &key.value,
            &token,
        );
        match decoded {
            Ok((access_token_expires, _)) => Ok(access_token_expires),
            Err(_err) => Err(CoreError::JwtInvalidOrExpired),
        }
    }

    pub fn decode_refresh_token(
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token: &str,
    ) -> CoreResult<String> {
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::RefreshToken,
            &key.value,
            &token,
        );
        match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| CoreError::CsrfNotFoundOrUsed),
            Err(_err) => Err(CoreError::JwtInvalidOrExpired),
        }
    }

    pub fn decode_csrf_key(
        service: &Service,
        user: &User,
        key: &KeyWithValue,
        token_type: JwtClaimsType,
        token: &str,
    ) -> CoreResult<Option<String>> {
        match Jwt::decode_token(service.id, user.id, token_type, &key.value, &token) {
            Ok((_, csrf_key)) => Ok(csrf_key),
            Err(_err) => Err(CoreError::JwtInvalidOrExpired),
        }
    }

    /// TOTP code verification.
    pub fn totp(key: &str, totp_code: &str) -> CoreResult<()> {
        let totp = TOTPBuilder::new()
            .base32_key(key)
            .finalize()
            .map_err(CoreError::libreauth_oath)?;

        if !totp.is_valid(&totp_code) {
            Err(CoreError::TotpInvalid)
        } else {
            Ok(())
        }
    }

    /// Authenticate root key.
    pub fn authenticate_root(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
    ) -> CoreResult<()> {
        match key_value {
            Some(key_value) => Key::read_by_root_value(driver, key_value)
                .map(|key| {
                    audit.key(Some(&key));
                    key
                })
                .map(|_key| ()),
            None => Err(CoreError::KeyUndefined),
        }
    }

    /// Authenticate service key.
    pub fn authenticate_service(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
    ) -> CoreResult<Service> {
        Auth::authenticate_service_try(driver, audit, key_value)
    }

    /// Authenticate service or root key.
    pub fn authenticate(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
    ) -> CoreResult<Option<Service>> {
        let key_value_1 = key_value.to_owned();

        Auth::authenticate_service_try(driver, audit, key_value)
            .map(Some)
            .or_else(move |_err| Auth::authenticate_root(driver, audit, key_value_1).map(|_| None))
    }

    fn authenticate_service_try(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
    ) -> CoreResult<Service> {
        match key_value {
            Some(key_value) => Key::read_by_service_value(driver, key_value)
                .and_then(|key| key.service_id.ok_or_else(|| CoreError::KeyServiceUndefined))
                .and_then(|service_id| Auth::authenticate_service_inner(driver, audit, service_id)),
            None => Err(CoreError::KeyUndefined),
        }
    }

    fn authenticate_service_inner(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        service_id: Uuid,
    ) -> CoreResult<Service> {
        let service = driver
            .service_read_opt(&ServiceRead::new(service_id))?
            .ok_or_else(|| CoreError::ServiceNotFound)?
            .check()?;
        audit.service(Some(&service));
        Ok(service)
    }

    /// Hash password string, none is returned if none is given as the input.
    /// <https://github.com/breard-r/libreauth>
    pub fn password_hash(password: Option<&str>) -> CoreResult<Option<String>> {
        match password {
            Some(password) => {
                let hasher = HashBuilder::new()
                    .version(USER_PASSWORD_HASH_VERSION)
                    .min_len(USER_PASSWORD_MIN_LEN)
                    .max_len(USER_PASSWORD_MAX_LEN)
                    .finalize()
                    .map_err(CoreError::libreauth_pass)?;

                let hashed = hasher.hash(password).map_err(CoreError::libreauth_pass)?;
                Ok(Some(hashed))
            }
            None => Ok(None),
        }
    }

    /// Check if password string and password hash match, an error is returned if they do not match or the hash is none.
    /// Returns true if the hash version does not match the current hash version.
    pub fn password_check(password_hash: Option<&str>, password: &str) -> CoreResult<bool> {
        match password_hash {
            Some(password_hash) => {
                let checker =
                    HashBuilder::from_phc(password_hash).map_err(CoreError::libreauth_pass)?;

                if checker.is_valid(password) {
                    Ok(checker.needs_update(Some(USER_PASSWORD_HASH_VERSION)))
                } else {
                    Err(CoreError::UserPasswordIncorrect)
                }
            }
            None => Err(CoreError::UserPasswordUndefined),
        }
    }

    /// Read user by ID.
    /// Checks user is enabled, returns bad request if disabled.
    pub fn user_read_by_id(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        audit: &mut AuditBuilder,
        id: Uuid,
    ) -> CoreResult<User> {
        let read = UserRead::Id(id);
        let user = User::read(driver, service_mask, &read)?;
        audit.user(Some(&user));
        if !user.is_enabled {
            return Err(CoreError::UserDisabled);
        }
        Ok(user)
    }

    /// Unchecked read user by ID.
    /// Does not check user is enabled.
    pub fn user_read_by_id_unchecked(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        audit: &mut AuditBuilder,
        id: Uuid,
    ) -> CoreResult<User> {
        let read = UserRead::Id(id);
        let user = User::read(driver, service_mask, &read)?;
        audit.user(Some(&user));
        Ok(user)
    }

    /// Read user by email address.
    /// Also checks user is enabled, returns bad request if disabled.
    pub fn user_read_by_email(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        audit: &mut AuditBuilder,
        email: String,
    ) -> CoreResult<User> {
        let read = UserRead::Email(email);
        let user = User::read(driver, service_mask, &read)?;
        audit.user(Some(&user));
        if !user.is_enabled {
            return Err(CoreError::UserDisabled);
        }
        Ok(user)
    }

    /// Read key by user reference and key type.
    /// Also checks key is enabled and not revoked, returns bad request if disabled.
    pub fn key_read_by_user(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        user: &User,
        key_type: KeyType,
    ) -> CoreResult<KeyWithValue> {
        let key = Key::read_by_user(driver, &service, &user, key_type)?;
        audit.user_key(Some(&key));
        if !key.is_enabled {
            Err(CoreError::KeyDisabled)
        } else if key.is_revoked {
            Err(CoreError::KeyRevoked)
        } else {
            Ok(key)
        }
    }

    /// Unchecked read key by user reference.
    /// Does not check key is enabled or not revoked.
    pub fn key_read_by_user_unchecked(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        user: &User,
        key_type: KeyType,
    ) -> CoreResult<KeyWithValue> {
        let key = Key::read_by_user(driver, &service, &user, key_type)?;
        audit.user_key(Some(&key));
        Ok(key)
    }

    /// Read key by user value.
    /// Also checks key is enabled and not revoked, returns bad request if disabled.
    pub fn key_read_by_user_value(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        key: String,
        key_type: KeyType,
    ) -> CoreResult<KeyWithValue> {
        let key = Key::read_by_user_value(driver, service, key, key_type)?;
        audit.user_key(Some(&key));
        if !key.is_enabled {
            Err(CoreError::KeyDisabled)
        } else if key.is_revoked {
            Err(CoreError::KeyRevoked)
        } else {
            Ok(key)
        }
    }

    /// Unchecked read key by user value.
    /// Does not check key is enabled and not revoked.
    pub fn key_read_by_user_value_unchecked(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        key: String,
        key_type: KeyType,
    ) -> CoreResult<KeyWithValue> {
        let key = Key::read_by_user_value(driver, service, key, key_type)?;
        audit.user_key(Some(&key));
        Ok(key)
    }

    /// Build user token by encoding access and refresh tokens.
    pub fn encode_user_token(
        driver: &dyn Driver,
        service: &Service,
        user: User,
        key: &KeyWithValue,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> CoreResult<UserToken> {
        let csrf = Auth::csrf_create(driver, service, refresh_token_expires)?;
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

    /// Create a new CSRF key, value pair using random key.
    pub fn csrf_create(
        driver: &dyn Driver,
        service: &Service,
        token_expires: i64,
    ) -> CoreResult<Csrf> {
        let csrf_key = Key::value_generate();
        Csrf::create(driver, service, csrf_key.clone(), csrf_key, token_expires)
    }

    /// Verify a CSRF key is valid by reading it, this will also delete the key.
    /// Also checks service verifying CSRF is same service that created it.
    pub fn csrf_verify(driver: &dyn Driver, service: &Service, csrf_key: String) -> CoreResult<()> {
        let res = Csrf::read_opt(driver, csrf_key)?.ok_or_else(|| CoreError::CsrfNotFoundOrUsed);

        match res {
            Ok(csrf) => {
                if csrf.service_id != service.id {
                    return Err(CoreError::CsrfNotFoundOrUsed);
                }
                Ok(())
            }
            Err(_err) => Err(CoreError::CsrfNotFoundOrUsed),
        }
    }
}
