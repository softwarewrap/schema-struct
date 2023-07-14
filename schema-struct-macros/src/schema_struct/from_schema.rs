use super::types::*;
use super::util::*;
use indexmap::IndexMap;
use serde_json::{Map, Value};
use std::collections::HashSet;

/// Parses a JSON schema.
pub trait FromSchema: Sized {
    /// Tries to parse a JSON schema into `Self`.
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, SchemaStructError>;
}

/// Implements `FromSchema` for a primitive JSON type.
macro_rules! impl_from_schema_primitive {
    ( $impl_ty:ty, $json_ty:literal ) => {
        impl FromSchema for $impl_ty {
            fn from_schema(
                value: &Value,
                _info: &mut FieldInfo,
            ) -> Result<Self, SchemaStructError> {
                assert_value_type(value, $json_ty)?;

                let default = value.get("default").map(ToOwned::to_owned);

                Ok(Self { default })
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
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, SchemaStructError> {
        assert_value_type(value, "array")?;

        get_prop_obj(value, "items")?.ok_or("array must have property `items`")?;
        let items_value = value
            .get("items")
            .ok_or("array must have property `items`")?;
        let mut items_info = FieldInfo {
            required: true,
            ..info.clone()
        };
        let items = Field::from_schema(items_value, &mut items_info)?;
        let default = value.get("default").map(ToOwned::to_owned);

        Ok(Self { items, default })
    }
}

impl FromSchema for ObjectField {
    fn from_schema(value: &Value, _info: &mut FieldInfo) -> Result<Self, SchemaStructError> {
        assert_value_type(value, "object")?;

        let empty_map = Map::new();
        let empty_vec = Vec::new();

        let properties = get_prop_obj(value, "properties")?.unwrap_or(&empty_map);
        let required_props_array = get_prop_array(value, "required")?.unwrap_or(&empty_vec);

        let required_props = required_props_array
            .iter()
            .map(|required_prop| {
                required_prop
                    .as_str()
                    .ok_or("required property names must be strings")
            })
            .collect::<Result<HashSet<_>, _>>()?;

        let fields = properties
            .iter()
            .map(|(property_name, property_value)| {
                let mut property_info = FieldInfo {
                    name: property_name.clone(),
                    description: None,
                    required: required_props.contains(property_name.as_str()),
                    subschema: false,
                };
                Field::from_schema(property_value, &mut property_info)
                    .map(|parsed_value| (property_name.clone(), parsed_value))
            })
            .collect::<Result<IndexMap<_, _>, _>>()?;

        let default = value.get("default").map(ToOwned::to_owned);

        Ok(Self { fields, default })
    }
}

impl FromSchema for EnumField {
    fn from_schema(value: &Value, _info: &mut FieldInfo) -> Result<Self, SchemaStructError> {
        let variant_values = get_prop_array(value, "enum")?.ok_or("no enum variants specified")?;

        let variants = variant_values
            .iter()
            .map(|variant| {
                variant
                    .as_str()
                    .map(|s| s.to_owned())
                    .ok_or("enum variants must be strings")
            })
            .collect::<Result<Vec<_>, _>>()?;

        let default = value.get("default").map(ToOwned::to_owned);

        Ok(Self { variants, default })
    }
}

impl FromSchema for TupleField {
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, SchemaStructError> {
        assert_value_type(value, "array")?;

        let tuple_items = get_prop_array(value, "prefixItems")?
            .ok_or("tuple must be defined using the `prefixItems` property")?;

        let items = tuple_items
            .iter()
            .enumerate()
            .map(|(index, tuple_item)| {
                let mut item_info = FieldInfo {
                    name: format!("{}{}", info.name, index),
                    description: None,
                    required: true,
                    subschema: false,
                };
                Field::from_schema(tuple_item, &mut item_info)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let default = value.get("default").map(ToOwned::to_owned);

        Ok(Self { items, default })
    }
}

impl FromSchema for RefField {
    fn from_schema(value: &Value, _info: &mut FieldInfo) -> Result<Self, SchemaStructError> {
        let ref_path = get_prop_str(value, "$ref")?.ok_or("refs must specify `$ref` property")?;
        let ty = RefType::from_path(ref_path)?;
        let default = value.get("default").map(ToOwned::to_owned);

        Ok(Self { ty, default })
    }
}

impl FromSchema for FieldType {
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, SchemaStructError> {
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
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, SchemaStructError> {
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
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, SchemaStructError> {
        Ok(Self {
            schema: Field::from_schema(value, info)?,
        })
    }
}
