//! # Notify Actor Messages
mod email_reset_password;
mod email_update_email;
mod email_update_password;

pub use crate::notify::notify_msg::{
    email_reset_password::EmailResetPassword, email_update_email::EmailUpdateEmail,
    email_update_password::EmailUpdatePassword,
};
