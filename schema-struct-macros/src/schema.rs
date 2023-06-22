use jsonschema::error::ValidationErrorKind;
use jsonschema::paths::JSONPointer;
use jsonschema::JSONSchema;
use serde_json::Value;
use std::ops::{Deref, DerefMut};

/// A validation error, modeled after `jsonschema::ValidationError`.
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

/// An error that can occur when parsing or validating a JSON schema.
#[derive(Debug)]
pub enum JsonSchemaError {
    /// The JSON schema isn't valid JSON.
    ParseError(serde_json::Error),
    /// The JSON schema isn't a valid schema.
    ValidationError(ValidationError),
}

/// A wrapper around `jsonschema::JSONSchema`.
pub struct JsonSchema(JSONSchema);

impl JsonSchema {
    /// Parses a JSON schema, checking the validity of the schema.
    pub fn parse(schema: &str) -> Result<Self, JsonSchemaError> {
        let schema_value: Value =
            serde_json::from_str(schema).map_err(JsonSchemaError::ParseError)?;

        let schema_parsed = JSONSchema::compile(&schema_value).map_err(|e| {
            JsonSchemaError::ValidationError(ValidationError {
                instance: schema_value.clone(),
                kind: e.kind,
                instance_path: e.instance_path,
                schema_path: e.schema_path,
            })
        })?;

        Ok(Self(schema_parsed))
    }
}

impl Deref for JsonSchema {
    type Target = JSONSchema;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for JsonSchema {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
