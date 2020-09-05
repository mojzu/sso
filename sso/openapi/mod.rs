mod structs;
mod traits;

pub use crate::{structs::*, traits::*};

pub use paperclip::v2::models::Reference;

use paperclip::v2::models::{Api, DataType, DataTypeFormat, Info, Resolvable};

// extern crate proc_macro;
// use proc_macro::TokenStream;
// #[proc_macro_derive(AnswerFn)]
// pub fn derive_answer_fn(_item: TokenStream) -> TokenStream {
//     "fn answer() -> u32 { 42 }".parse().unwrap()
// }

#[derive(Debug, Clone, Default)]
pub struct Builder {
    api: Api<Parameter, Response, Schema>,
}

impl Builder {
    pub fn new(info_version: &str, info_title: &str) -> Self {
        let mut info = Info::default();
        info.version = info_version.to_string();
        info.title = info_title.to_string();

        let mut api = Api::default();
        api.info = info;

        Self { api }
    }

    pub fn definition<D: Definition>(mut self, definition: D) -> Self {
        self.api
            .definitions
            .entry(definition.name())
            .or_insert(definition.schema());
        self
    }

    pub fn path<I: IntoPathItem>(mut self, path: &str, item: I) -> Self {
        self.api
            .paths
            .entry(path.to_string())
            .or_insert(item.into_path_item());
        self
    }

    pub fn json_value(&self) -> serde_json::Value {
        serde_json::from_str(&self.serialise_json()).expect("failed to get json value")
    }

    pub fn serialise_json(&self) -> String {
        serde_json::to_string(&self.api).expect("failed to serialise api to json format")
    }

    pub fn serialise_yaml(&self) -> String {
        serde_yaml::to_string(&self.api).expect("failed to serialise api to yaml format")
    }
}

#[derive(Debug, Clone, Default)]
pub struct SchemaBuilder {
    schema: Schema,
}

impl SchemaBuilder {
    pub fn object() -> Self {
        let mut schema = Schema::default();
        schema.data_type = Some(DataType::Object);
        Self { schema }
    }

    pub fn type_bool() -> Self {
        let mut schema = Schema::default();
        schema.data_type = Some(DataType::Boolean);
        Self { schema }
    }

    pub fn type_i32() -> Self {
        let mut schema = Schema::default();
        schema.data_type = Some(DataType::Integer);
        schema.format = Some(DataTypeFormat::Int32);
        Self { schema }
    }

    pub fn type_i64() -> Self {
        let mut schema = Schema::default();
        schema.data_type = Some(DataType::Integer);
        schema.format = Some(DataTypeFormat::Int64);
        Self { schema }
    }

    pub fn type_string() -> Self {
        let mut schema = Schema::default();
        schema.data_type = Some(DataType::String);
        Self { schema }
    }

    pub fn type_datetime() -> Self {
        let mut schema = Schema::default();
        schema.data_type = Some(DataType::String);
        schema.format = Some(DataTypeFormat::DateTime);
        Self { schema }
    }

    pub fn type_array(items: Schema) -> Self {
        let mut schema = Schema::default();
        schema.data_type = Some(DataType::Array);
        schema.items = Some(Resolvable::from(items));
        Self { schema }
    }

    pub fn reference<D: Definition>(definition: D) -> Self {
        let mut schema = Schema::default();
        schema.reference = Some(format!("#/definitions/{}", definition.name()));
        Self { schema }
    }

    pub fn description(mut self, description: &str) -> Self {
        self.schema.description = Some(description.to_string());
        self
    }

    pub fn example(mut self, example: serde_json::Value) -> Self {
        self.schema.example = Some(example);
        self
    }

    pub fn enum_variants(mut self, variants: Vec<serde_json::Value>) -> Self {
        self.schema.enum_ = variants;
        self
    }

    pub fn property(mut self, name: &str, schema: Schema) -> Self {
        self.schema
            .properties
            .entry(name.to_string())
            .or_insert(Resolvable::from(schema));
        self
    }

    pub fn property_required(mut self, name: &str, schema: Schema) -> Self {
        self.schema.required.insert(name.to_string());
        self.property(name, schema)
    }

    pub fn schema(self) -> Schema {
        self.schema
    }
}

pub trait Definition {
    type Object: serde::ser::Serialize + serde::de::DeserializeOwned;

    fn name(&self) -> String;
    fn schema(&self) -> Schema;

    fn reference(&self) -> Reference {
        Reference {
            reference: format!("#/definitions/{}", self.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Schema, SchemaBuilder};
    use chrono::prelude::*;

    #[derive(Apiv2Schema, Serialize, Deserialize)]
    struct ApiResponse {
        code: Option<i32>,
        #[serde(rename = "type")]
        type_: Option<String>,
        message: Option<String>,
    }
    struct ApiResponseDefinition;
    impl super::Definition for ApiResponseDefinition {
        type Object = ApiResponse;

        fn name(&self) -> String {
            "ApiResponse".to_string()
        }
        fn schema(&self) -> Schema {
            SchemaBuilder::object()
                .property("code", SchemaBuilder::type_i32().schema())
                .property("type", SchemaBuilder::type_string().schema())
                .property("message", SchemaBuilder::type_string().schema())
                .schema()
        }
    }

    #[derive(Serialize, Deserialize)]
    struct Category {
        id: Option<i64>,
        name: Option<String>,
    }
    struct CategoryDefinition;
    impl super::Definition for CategoryDefinition {
        type Object = Category;

        fn name(&self) -> String {
            "Category".to_string()
        }
        fn schema(&self) -> Schema {
            SchemaBuilder::object()
                .property("id", SchemaBuilder::type_i64().schema())
                .property("name", SchemaBuilder::type_string().schema())
                .schema()
        }
    }

    #[derive(Serialize, Deserialize)]
    struct Tag {
        id: Option<i64>,
        name: Option<String>,
    }
    struct TagDefinition;
    impl super::Definition for TagDefinition {
        type Object = Tag;

        fn name(&self) -> String {
            "Tag".to_string()
        }
        fn schema(&self) -> Schema {
            SchemaBuilder::object()
                .property("id", SchemaBuilder::type_i64().schema())
                .property("name", SchemaBuilder::type_string().schema())
                .schema()
        }
    }

    #[derive(Serialize, Deserialize)]
    struct Pet {
        id: Option<i64>,
        category: Option<Category>,
        name: String,
        #[serde(rename = "photoUrls")]
        photo_urls: Vec<String>,
        tags: Option<Vec<Tag>>,
        status: Option<String>,
    }
    struct PetDefinition;
    impl super::Definition for PetDefinition {
        type Object = Pet;

        fn name(&self) -> String {
            "Pet".to_string()
        }
        fn schema(&self) -> Schema {
            SchemaBuilder::object()
                .property("id", SchemaBuilder::type_i64().schema())
                .property(
                    "category",
                    SchemaBuilder::reference(CategoryDefinition).schema(),
                )
                .property_required(
                    "name",
                    SchemaBuilder::type_string()
                        .example(json!("doggie"))
                        .schema(),
                )
                .property_required(
                    "photoUrls",
                    SchemaBuilder::type_array(SchemaBuilder::type_string().schema()).schema(),
                )
                .property(
                    "tags",
                    SchemaBuilder::type_array(SchemaBuilder::reference(TagDefinition).schema())
                        .schema(),
                )
                .property(
                    "status",
                    SchemaBuilder::type_string()
                        .description("pet status in the store")
                        .enum_variants(vec![json!("available"), json!("pending"), json!("sold")])
                        .schema(),
                )
                .schema()
        }
    }

    #[derive(Serialize, Deserialize)]
    struct Order {
        id: Option<i64>,
        #[serde(rename = "petId")]
        pet_id: Option<i64>,
        quantity: Option<i32>,
        #[serde(rename = "shipDate")]
        ship_date: Option<DateTime<FixedOffset>>,
        status: Option<String>,
        complete: Option<bool>,
    }
    struct OrderDefinition;
    impl super::Definition for OrderDefinition {
        type Object = Order;

        fn name(&self) -> String {
            "Order".to_string()
        }
        fn schema(&self) -> Schema {
            SchemaBuilder::object()
                .property("id", SchemaBuilder::type_i64().schema())
                .property("petId", SchemaBuilder::type_i64().schema())
                .property("quantity", SchemaBuilder::type_i32().schema())
                .property("shipDate", SchemaBuilder::type_datetime().schema())
                .property(
                    "status",
                    SchemaBuilder::type_string()
                        .description("Order Status")
                        .enum_variants(vec![json!("placed"), json!("approved"), json!("delivered")])
                        .schema(),
                )
                .property("complete", SchemaBuilder::type_bool().schema())
                .schema()
        }
    }

    #[derive(Serialize, Deserialize)]
    struct User {
        id: Option<i64>,
        username: Option<String>,
        #[serde(rename = "firstName")]
        first_name: Option<String>,
        #[serde(rename = "lastName")]
        last_name: Option<String>,
        email: Option<String>,
        password: Option<String>,
        phone: Option<String>,
        #[serde(rename = "userStatus")]
        user_status: Option<i32>,
    }
    struct UserDefinition;
    impl super::Definition for UserDefinition {
        type Object = User;

        fn name(&self) -> String {
            "User".to_string()
        }
        fn schema(&self) -> Schema {
            SchemaBuilder::object()
                .property("id", SchemaBuilder::type_i64().schema())
                .property("username", SchemaBuilder::type_string().schema())
                .property("firstName", SchemaBuilder::type_string().schema())
                .property("lastName", SchemaBuilder::type_string().schema())
                .property("email", SchemaBuilder::type_string().schema())
                .property("password", SchemaBuilder::type_string().schema())
                .property("phone", SchemaBuilder::type_string().schema())
                .property(
                    "userStatus",
                    SchemaBuilder::type_i32()
                        .description("User Status")
                        .schema(),
                )
                .schema()
        }
    }

    #[test]
    fn petstore_swagger() {
        let b = super::Builder::new("v1", "example")
            .definition(ApiResponseDefinition)
            .definition(CategoryDefinition)
            .definition(PetDefinition)
            .definition(TagDefinition)
            .definition(OrderDefinition)
            .definition(UserDefinition);

        let o = b.json_value();
        let definitions = &o["definitions"];

        assert_eq!(
            definitions["ApiResponse"],
            json!({
                "type": "object",
                "properties": {
                    "code": { "type": "integer", "format": "int32" },
                    "type": { "type": "string" },
                    "message": { "type": "string" },
                },
            })
        );
        assert_eq!(
            definitions["Category"],
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer", "format": "int64" },
                    "name": { "type": "string" },
                },
            })
        );
        assert_eq!(
            definitions["Pet"],
            json!({
                "type": "object",
                "required": ["name", "photoUrls"],
                "properties": {
                    "id": { "type": "integer", "format": "int64" },
                    "category": { "$ref": "#/definitions/Category" },
                    "name": { "type": "string", "example": "doggie" },
                    "photoUrls": { "type": "array", "items": { "type": "string" } },
                    "tags": { "type": "array", "items": { "$ref": "#/definitions/Tag" } },
                    "status": { "type": "string", "description": "pet status in the store", "enum": ["available", "pending", "sold"] },
                },
            })
        );
        assert_eq!(
            definitions["Tag"],
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer", "format": "int64" },
                    "name": { "type": "string" },
                },
            })
        );
        assert_eq!(
            definitions["Order"],
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer", "format": "int64" },
                    "petId": { "type": "integer", "format": "int64" },
                    "quantity": { "type": "integer", "format": "int32" },
                    "shipDate": { "type": "string", "format": "date-time" },
                    "status": { "type": "string", "description": "Order Status", "enum": ["placed", "approved", "delivered"] },
                    "complete": { "type": "boolean" },
                },
            })
        );
        assert_eq!(
            definitions["User"],
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer", "format": "int64" },
                    "username": { "type": "string" },
                    "firstName": { "type": "string" },
                    "lastName": { "type": "string" },
                    "email": { "type": "string" },
                    "password": { "type": "string" },
                    "phone": { "type": "string" },
                    "userStatus": { "type": "integer", "format": "int32", "description": "User Status" },
                },
            }),
        );

        println!("{}", b.serialise_yaml());

        unimplemented!();
    }
}
