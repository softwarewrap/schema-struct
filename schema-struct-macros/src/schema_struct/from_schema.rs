use super::types::*;
use super::util::*;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

/// Parses a JSON schema.
pub trait FromSchema: Sized {
    /// Tries to parse a JSON schema into `Self`.
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, FromSchemaError>;
}

/// Implements `FromSchema` for a primitive JSON type.
macro_rules! impl_from_schema_primitive {
    ( $impl_ty:ty, $json_ty:literal ) => {
        impl FromSchema for $impl_ty {
            fn from_schema(value: &Value, _info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
                assert_value_type(value, $json_ty)?;

                Ok(Self)
            }
        }
    };
}

impl_from_schema_primitive!(NullField, "null");
impl_from_schema_primitive!(BooleanField, "boolean");
impl_from_schema_primitive!(IntegerField, "integer");
impl_from_schema_primitive!(NumberField, "number");
impl_from_schema_primitive!(StringField, "string");

impl FromSchema for ArrayField {
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        assert_value_type(value, "array")?;

        get_prop_obj(value, "items")?.ok_or("array must have property `items`")?;
        let items_value = value
            .get("items")
            .ok_or("array must have property `items`")?;
        let items = Field::from_schema(items_value, info)?;

        Ok(Self { items })
    }
}

impl FromSchema for ObjectField {
    fn from_schema(value: &Value, _info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        assert_value_type(value, "object")?;

        let empty_map = Map::new();
        let empty_vec = Vec::new();

        let properties = get_prop_obj(value, "properties")?.unwrap_or(&empty_map);
        let required_props_array = get_prop_array(value, "required")?.unwrap_or(&empty_vec);
        let mut required_props = HashSet::new();

        for required_prop in required_props_array {
            let prop_name = required_prop
                .as_str()
                .ok_or("required property names must be strings")?;
            required_props.insert(prop_name);
        }

        let mut fields = HashMap::new();

        for (property_name, property_value) in properties {
            let mut property_info = FieldInfo {
                name: property_name.clone(),
                description: None,
                required: required_props.contains(property_name.as_str()),
                subschema: false,
            };
            let parsed_value = Field::from_schema(property_value, &mut property_info)?;
            fields.insert(property_name.to_owned(), parsed_value);
        }

        Ok(Self { fields })
    }
}

impl FromSchema for EnumField {
    fn from_schema(value: &Value, _info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        let variant_values = get_prop_array(value, "enum")?.ok_or("no enum variants specified")?;

        let mut variants = Vec::new();

        for variant in variant_values {
            let variant_name = variant.as_str().ok_or("enum variants must be strings")?;
            variants.push(variant_name.to_owned());
        }

        Ok(Self { variants })
    }
}

impl FromSchema for TupleField {
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        assert_value_type(value, "array")?;

        let tuple_items = get_prop_array(value, "prefixItems")?
            .ok_or("tuple must be defined using the `prefixItems` property")?;

        let mut items = Vec::new();

        for (index, tuple_item) in tuple_items.iter().enumerate() {
            let mut item_info = FieldInfo {
                name: format!("{}{}", info.name, index),
                description: None,
                required: true,
                subschema: false,
            };
            let parsed_value = Field::from_schema(tuple_item, &mut item_info)?;
            items.push(parsed_value);
        }

        Ok(Self { items })
    }
}

impl FromSchema for RefField {
    fn from_schema(value: &Value, _info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        let ref_path = get_prop_str(value, "$ref")?.ok_or("refs must specify `$ref` property")?;
        let path = ref_path.split('/').map(|s| s.to_owned()).collect();

        Ok(Self { path })
    }
}

impl FromSchema for FieldType {
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        Ok(match parse_value_type(value)? {
            ValueType::Null => Self::Null(NullField::from_schema(value, info)?),
            ValueType::Boolean => Self::Boolean(BooleanField::from_schema(value, info)?),
            ValueType::Integer => Self::Integer(IntegerField::from_schema(value, info)?),
            ValueType::Number => Self::Number(NumberField::from_schema(value, info)?),
            ValueType::String => Self::String(StringField::from_schema(value, info)?),
            ValueType::Array => Self::Array(ArrayField::from_schema(value, info)?),
            ValueType::Object => Self::Object(ObjectField::from_schema(value, info)?),
            ValueType::Enum => Self::Enum(EnumField::from_schema(value, info)?),
            ValueType::Tuple => Self::Tuple(TupleField::from_schema(value, info)?),
            ValueType::Ref => Self::Ref(RefField::from_schema(value, info)?),
        })
    }
}

impl FromSchema for Field {
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        let description = get_prop_str(value, "description")?.map(|s| s.to_owned());
        let mut field_info = FieldInfo {
            description,
            ..info.clone()
        };
        let field_ty = FieldType::from_schema(value, &mut field_info)?;

        Ok(Self {
            info: field_info,
            ty: Box::new(field_ty),
        })
    }
}

impl FromSchema for Subschema {
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        Ok(Self {
            schema: Field::from_schema(value, info)?,
        })
    }
}
