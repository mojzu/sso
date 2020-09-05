use paperclip_macros::api_v2_schema_struct;

#[derive(Debug, Clone, Default, Serialize)]
pub struct Parameter {}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Response {}

#[api_v2_schema_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
}
