use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};

pub fn canonical_json(value: &Value) -> String {
    let sorted = sort_value(value);
    serde_json::to_string(&sorted).expect("canonical JSON serialization")
}

fn sort_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for key in keys {
                sorted.insert(key.clone(), sort_value(&map[key]));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.iter().map(sort_value).collect()),
        _ => value.clone(),
    }
}

pub fn sha256_bytes(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn sha256_canonical(value: &Value) -> String {
    sha256_bytes(canonical_json(value).as_bytes())
}

pub fn sha256_canonical_line(value: &Value) -> String {
    sha256_bytes(format!("{}\n", canonical_json(value)).as_bytes())
}

pub fn canonical_json_line(value: &Value) -> String {
    format!("{}\n", canonical_json(value))
}

pub fn parse_json_strict(text: &str) -> Result<Value, String> {
    let mut de = serde_json::Deserializer::from_str(text);
    let value = StrictValueSeed
        .deserialize(&mut de)
        .map_err(|error| error.to_string())?;
    de.end().map_err(|error| error.to_string())?;
    Ok(value)
}

struct StrictValueSeed;

impl<'de> DeserializeSeed<'de> for StrictValueSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NoDupVisitor)
    }
}

struct NoDupVisitor;

impl<'de> Visitor<'de> for NoDupVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(v.into()))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(v.into()))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        serde_json::Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| de::Error::custom("invalid JSON number"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
        Ok(Value::String(v.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
        Ok(Value::String(v))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(StrictValueSeed)? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut object = Map::new();
        let mut seen = HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !seen.insert(key.clone()) {
                return Err(de::Error::custom(format!("duplicate JSON key: {key}")));
            }
            let value = map.next_value_seed(StrictValueSeed)?;
            object.insert(key, value);
        }
        Ok(Value::Object(object))
    }
}

pub fn parse_json_bytes_strict(bytes: &[u8]) -> Result<Value, String> {
    let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
    parse_json_strict(text)
}

pub fn is_safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains('\\')
        && !path.split('/').any(|part| part == "..")
}

pub fn duplicate_values(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut dupes = Vec::new();
    for value in values {
        if !seen.insert(value.clone()) && !dupes.contains(value) {
            dupes.push(value.clone());
        }
    }
    dupes
}

pub fn object_keys(value: &Value) -> Option<Vec<&str>> {
    value.as_object().map(|map| {
        let mut keys: Vec<_> = map.keys().map(String::as_str).collect();
        keys.sort();
        keys
    })
}

pub fn values_equal(left: &Value, right: &Value) -> bool {
    left == right
}

pub fn get_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

pub fn require_str(value: &Value, key: &str) -> Result<String, String> {
    get_str(value, key)
        .map(str::to_owned)
        .ok_or_else(|| format!("missing or invalid string field: {key}"))
}

pub fn require_u64(value: &Value, key: &str) -> Result<u64, String> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing or invalid integer field: {key}"))
}

pub fn require_str_array(value: &Value, key: &str) -> Result<Vec<String>, String> {
    let array = value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing or invalid array field: {key}"))?;
    array
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("non-string entry in {key}"))
        })
        .collect()
}

pub fn has_exact_keys(value: &Value, keys: &[&str]) -> bool {
    value
        .as_object()
        .is_some_and(|map| map.len() == keys.len() && keys.iter().all(|key| map.contains_key(*key)))
}

pub fn as_object_map(value: &Value) -> Result<&Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| "expected JSON object".to_string())
}

pub fn index_by_id(entries: &[Value], id_field: &str) -> Result<HashMap<String, Value>, String> {
    let mut map = HashMap::new();
    for entry in entries {
        let id = require_str(entry, id_field)?;
        if map.insert(id.clone(), entry.clone()).is_some() {
            return Err(format!("duplicate id: {id}"));
        }
    }
    Ok(map)
}
