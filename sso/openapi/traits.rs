use crate::structs::{Parameter, Response, Schema};
use paperclip::v2::models::PathItem;

pub trait IntoPathItem {
    fn into_path_item(self) -> PathItem<Parameter, Response>;
}

pub trait IntoSchema {
    fn into_schema() -> Schema;
}
