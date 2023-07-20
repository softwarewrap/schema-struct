use jsonschema::error::ValidationErrorKind;
use jsonschema::paths::JSONPointer;
use jsonschema::JSONSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A schema validation error, modeled after `jsonschema::ValidationError`.
#[derive(Debug)]
pub struct ValidationError {
    /// Value of the property that failed validation.
    pub instance: Value,
    /// Type of validation error.
    pub kind: ValidationErrorKind,
    /// Path to the value that failed validation.
    pub instance_path: JSONPointer,
    /// Path to the JSON Schema keyword that failed validation.
    pub schema_path: JSONPointer,
}

impl From<jsonschema::ValidationError<'_>> for ValidationError {
    fn from(value: jsonschema::ValidationError<'_>) -> Self {
        Self {
            instance: value.instance.into_owned(),
            kind: value.kind,
            instance_path: value.instance_path,
            schema_path: value.schema_path,
        }
    }
}

/// An error that can occur when parsing or validating a JSON value.
#[derive(Debug)]
pub enum JsonSchemaError {
    /// The JSON value failed to parse.
    SerializeDeserializeError(serde_json::Error),
    /// The JSON schema isn't a valid schema.
    SchemaError(ValidationError),
    /// The JSON value doesn't match the schema.
    SchemaValidationError(Vec<ValidationError>),
}

impl From<serde_json::Error> for JsonSchemaError {
    fn from(value: serde_json::Error) -> Self {
        Self::SerializeDeserializeError(value)
    }
}

impl From<jsonschema::ValidationError<'_>> for JsonSchemaError {
    fn from(value: jsonschema::ValidationError) -> Self {
        Self::SchemaError(value.into())
    }
}

impl<'a> From<Box<dyn Iterator<Item = jsonschema::ValidationError<'a>> + Sync + Send + 'a>>
    for JsonSchemaError
{
    fn from(
        value: Box<dyn Iterator<Item = jsonschema::ValidationError<'a>> + Sync + Send + 'a>,
    ) -> Self {
        Self::SchemaValidationError(value.map(|e| e.into()).collect())
    }
}

/// A generic JSON schema error.
pub type Result<T> = core::result::Result<T, JsonSchemaError>;

/// Serializes a type to a JSON string.
pub fn serialize<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    Ok(serde_json::to_string(&value)?)
}

/// Deserializes a JSON string into a type.
pub fn deserialize<'a, T>(json: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    Ok(serde_json::from_str(json)?)
}

/// Deserializes a JSON string into a type and validates it against a JSON
/// schema.
pub fn deserialize_validate<'a, T>(json: &'a str, schema: &str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let schema_value: Value = serde_json::from_str(schema)?;
    let json_value: Value = serde_json::from_str(json)?;
    JSONSchema::compile(&schema_value)?.validate(&json_value)?;
    deserialize(json)
}

/// Serializes a type to a JSON value.
pub fn serialize_to_value<T>(value: &T) -> Result<Value>
where
    T: ?Sized + Serialize,
{
    Ok(serde_json::to_value(value)?)
}

/// Deserializes a JSON value into a type.
pub fn deserialize_from_value<T>(value: Value) -> Result<T>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_value(value)?)
}

/// Deserializes a JSON string into a type and validates it against a JSON
/// schema.
pub fn deserialize_from_value_validate<T>(value: Value, schema: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let schema_value: Value = serde_json::from_str(schema)?;
    JSONSchema::compile(&schema_value)?.validate(&value)?;
    deserialize_from_value(value)
}
