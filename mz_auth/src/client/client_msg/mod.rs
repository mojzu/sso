//! # Client Actor Messages
mod delete;
mod get;
mod patch_json;
mod post_json;

pub use crate::client::client_msg::{
    delete::Delete, get::Get, patch_json::PatchJson, post_json::PostJson,
};
