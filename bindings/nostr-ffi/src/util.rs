// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use nostr::serde_json::{Number, Value};
use nostr::util;
use uniffi::Enum;

use crate::error::Result;
use crate::{NostrError, PublicKey, SecretKey};

#[uniffi::export]
pub fn generate_shared_key(secret_key: Arc<SecretKey>, public_key: Arc<PublicKey>) -> Vec<u8> {
    util::generate_shared_key(secret_key.as_ref().deref(), public_key.as_ref().deref()).to_vec()
}

#[derive(Enum)]
pub enum JsonValue {
    Bool { bool: bool },
    NumberPosInt { number: u64 },
    NumberNegInt { number: i64 },
    NumberFloat { number: f64 },
    Str { s: String },
    Array { array: Vec<JsonValue> },
    Object { map: HashMap<String, JsonValue> },
    Null(),
}

impl TryFrom<JsonValue> for Value {
    type Error = NostrError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        Ok(match value {
            JsonValue::Bool { bool } => Self::Bool(bool),
            JsonValue::NumberPosInt { number } => Self::Number(Number::from(number)),
            JsonValue::NumberNegInt { number } => Self::Number(Number::from(number)),
            JsonValue::NumberFloat { number } => {
                let float = Number::from_f64(number).ok_or(NostrError::Generic {
                    err: String::from("Impossible to convert finite f64 to number"),
                })?;
                Self::Number(float)
            }
            JsonValue::Str { s } => Self::String(s),
            JsonValue::Array { array } => Self::Array(
                array
                    .into_iter()
                    .filter_map(|v| v.try_into().ok())
                    .collect(),
            ),
            JsonValue::Object { map } => Self::Object(
                map.into_iter()
                    .filter_map(|(k, v)| Some((k, v.try_into().ok()?)))
                    .collect(),
            ),
            JsonValue::Null() => Self::Null,
        })
    }
}
