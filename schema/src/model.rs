use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Any {
    Boolean(bool),
    Number(i32),
    String(String),
    Array(Vec<Any>),
    Object(HashMap<String, Any>),
    Null,
}

// ---------------------------------------------------------------------------

#[derive(Debug, Default, PartialEq)]
pub struct Extensions {
    pub values: HashMap<String, Any>,
}

#[derive(Default)]
struct ExtensionsVisitor;

impl<'de> Visitor<'de> for ExtensionsVisitor {
    type Value = Extensions;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("OpenAPI specification extensions")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = Extensions::default();
        while let Some(key) = access.next_key::<String>()? {
            if key.starts_with("x-") {
                let value = access.next_value()?;
                map.values
                    .insert(key.strip_prefix("x-").unwrap().to_string(), value);
            }
        }
        Ok(map)
    }
}

impl Serialize for Extensions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.values.len()))?;
        for (k, v) in &self.values {
            map.serialize_entry(&format!("x-{k}"), v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Extensions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ExtensionsVisitor)
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct HttpStatuses<T> {
    pub values: HashMap<u16, T>,
}

impl<T> Default for HttpStatuses<T> {
    fn default() -> Self {
        HttpStatuses::<T> {
            values: HashMap::new(),
        }
    }
}

struct HttpStatusesVisitor<T> {
    marker: PhantomData<HttpStatuses<T>>,
}

impl<T> Default for HttpStatusesVisitor<T> {
    fn default() -> Self {
        HttpStatusesVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for HttpStatusesVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = HttpStatuses<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("OpenAPI responses object")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = HttpStatuses::<T>::default();
        while let Some(key) = access.next_key::<String>()? {
            if let Ok(k) = u16::from_str(&key) {
                let value = access.next_value()?;
                map.values.insert(k, value);
            }
        }
        Ok(map)
    }
}

impl<T> Serialize for HttpStatuses<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.values.len()))?;
        for (k, v) in &self.values {
            map.serialize_entry(&k.to_string(), v)?;
        }
        map.end()
    }
}

impl<'de, T> Deserialize<'de> for HttpStatuses<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(HttpStatusesVisitor::default())
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct KeyValues<T> {
    pub values: HashMap<String, T>,
    pub extensions: HashMap<String, Any>,
}

impl<T> Default for KeyValues<T> {
    fn default() -> Self {
        KeyValues::<T> {
            values: HashMap::new(),
            extensions: HashMap::new(),
        }
    }
}

struct KeyValuesVisitor<T> {
    marker: PhantomData<KeyValues<T>>,
}

impl<T> Default for KeyValuesVisitor<T> {
    fn default() -> Self {
        KeyValuesVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for KeyValuesVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = KeyValues<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("OpenAPI key and value with specification extensions")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = KeyValues::default();
        while let Some(key) = access.next_key::<String>()? {
            if key.starts_with("x-") {
                let value = access.next_value()?;
                map.extensions
                    .insert(key.strip_prefix("x-").unwrap().to_string(), value);
            } else {
                let value = access.next_value()?;
                map.values.insert(key, value);
            }
        }
        Ok(map)
    }
}

impl<T> Serialize for KeyValues<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.values.len() + self.extensions.len()))?;
        for (k, v) in &self.values {
            map.serialize_entry(k, v)?;
        }
        for (k, v) in &self.extensions {
            map.serialize_entry(&format!("x-{k}"), v)?;
        }
        map.end()
    }
}

impl<'de, T> Deserialize<'de> for KeyValues<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(KeyValuesVisitor::default())
    }
}
