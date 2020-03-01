mod client;
mod method;
mod options;
mod server;
mod util;

pub use crate::grpc::{client::*, options::*, server::*, util::*};

pub mod pb {
    //! Generated protobuf server and client items.
    tonic::include_proto!("sso");

    use crate::KeyType as DriverKeyType;
    use chrono::{DateTime, Utc};
    use std::convert::TryInto;
    use uuid::Uuid;

    pub fn timestamp_opt_to_datetime_opt(
        ti: Option<prost_types::Timestamp>,
    ) -> Option<DateTime<Utc>> {
        match ti {
            Some(ti) => {
                let st: std::time::SystemTime = ti.try_into().unwrap();
                let dt: DateTime<Utc> = st.into();
                Some(dt)
            }
            None => None,
        }
    }

    pub fn timestamp_opt_to_datetime(ti: Option<prost_types::Timestamp>) -> DateTime<Utc> {
        timestamp_opt_to_datetime_opt(ti).unwrap()
    }

    pub fn datetime_to_timestamp_opt(dt: DateTime<Utc>) -> Option<prost_types::Timestamp> {
        let st: std::time::SystemTime = dt.into();
        let ti: prost_types::Timestamp = st.into();
        Some(ti)
    }

    pub fn datetime_opt_to_timestamp_opt(
        dt: Option<DateTime<Utc>>,
    ) -> Option<prost_types::Timestamp> {
        match dt {
            Some(dt) => datetime_to_timestamp_opt(dt),
            None => None,
        }
    }

    pub fn string_to_uuid(s: String) -> Uuid {
        Uuid::parse_str(s.as_ref()).unwrap()
    }

    pub fn string_opt_to_uuid_opt(s: Option<String>) -> Option<Uuid> {
        match s {
            Some(s) => {
                let u: Uuid = Uuid::parse_str(s.as_ref()).unwrap();
                Some(u)
            }
            None => None,
        }
    }

    pub fn string_vec_to_uuid_vec_opt(s: Vec<String>) -> Option<Vec<Uuid>> {
        if s.is_empty() {
            None
        } else {
            Some(
                s.into_iter()
                    .map(|s| Uuid::parse_str(s.as_ref()).unwrap())
                    .collect(),
            )
        }
    }

    pub fn string_vec_to_string_vec_opt(s: Vec<String>) -> Option<Vec<String>> {
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }

    pub fn i32_vec_to_key_type_vec_opt(s: Vec<i32>) -> Option<Vec<DriverKeyType>> {
        if s.is_empty() {
            None
        } else {
            Some(s.into_iter().map(DriverKeyType::from_i32).collect())
        }
    }

    pub fn key_type_vec_opt_to_i32_vec(s: Option<Vec<DriverKeyType>>) -> Vec<i32> {
        match s {
            Some(s) => s.into_iter().map(|x| x as i32).collect(),
            None => Vec::new(),
        }
    }

    pub fn uuid_to_string(u: Uuid) -> String {
        format!("{}", u)
    }

    pub fn uuid_opt_to_string_opt(u: Option<Uuid>) -> Option<String> {
        match u {
            Some(u) => Some(uuid_to_string(u)),
            None => None,
        }
    }

    pub fn uuid_vec_opt_to_string_vec(u: Option<Vec<Uuid>>) -> Vec<String> {
        match u {
            Some(u) => u
                .into_iter()
                .map::<String, _>(|x| format!("{}", x))
                .collect(),
            None => Vec::new(),
        }
    }

    fn struct_kind_to_value(s: prost_types::value::Kind) -> serde_json::Value {
        match s {
            prost_types::value::Kind::NullValue(_x) => serde_json::Value::Null,
            prost_types::value::Kind::NumberValue(x) => {
                let n = serde_json::Number::from_f64(x);
                serde_json::Value::Number(n.unwrap())
            }
            prost_types::value::Kind::StringValue(x) => serde_json::Value::String(x),
            prost_types::value::Kind::BoolValue(x) => serde_json::Value::Bool(x),
            prost_types::value::Kind::StructValue(x) => struct_opt_to_value_opt(Some(x)).unwrap(),
            prost_types::value::Kind::ListValue(x) => {
                let mut v = Vec::new();
                for value in x.values {
                    if let Some(kind) = value.kind {
                        v.push(struct_kind_to_value(kind))
                    }
                }
                serde_json::Value::Array(v)
            }
        }
    }

    pub fn struct_to_value(s: prost_types::Struct) -> serde_json::Value {
        let mut m = serde_json::Map::default();
        for (key, value) in s.fields {
            if let Some(kind) = value.kind {
                m.insert(key, struct_kind_to_value(kind));
            }
        }
        serde_json::Value::Object(m)
    }

    pub fn struct_opt_to_value_opt(s: Option<prost_types::Struct>) -> Option<serde_json::Value> {
        match s {
            Some(s) => Some(struct_to_value(s)),
            None => None,
        }
    }

    fn value_to_struct_value(value: serde_json::Value) -> prost_types::Value {
        let kind: prost_types::value::Kind = match value {
            serde_json::Value::Null => prost_types::value::Kind::NullValue(0),
            serde_json::Value::Bool(x) => prost_types::value::Kind::BoolValue(x),
            serde_json::Value::Number(x) => {
                prost_types::value::Kind::NumberValue(x.as_f64().unwrap())
            }
            serde_json::Value::String(x) => prost_types::value::Kind::StringValue(x),
            serde_json::Value::Array(x) => {
                let mut v = Vec::new();
                for value in x {
                    v.push(value_to_struct_value(value));
                }
                prost_types::value::Kind::ListValue(prost_types::ListValue { values: v })
            }
            serde_json::Value::Object(x) => {
                let mut fields = std::collections::BTreeMap::new();
                for (key, value) in x {
                    fields.insert(key, value_to_struct_value(value));
                }
                prost_types::value::Kind::StructValue(prost_types::Struct { fields })
            }
        };
        prost_types::Value { kind: Some(kind) }
    }

    pub fn value_to_struct_opt(s: serde_json::Value) -> Option<prost_types::Struct> {
        let mut fields = std::collections::BTreeMap::new();
        match s {
            serde_json::Value::Object(x) => {
                for (key, value) in x {
                    fields.insert(key, value_to_struct_value(value));
                }
                Some(prost_types::Struct { fields })
            }
            _ => None,
        }
    }
}
