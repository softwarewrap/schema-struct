use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use regex::Regex;
use serde_json::{Map, Value};

/// A JSON value type.
#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    Null,
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
    Enum,
    Tuple,
    Ref,
}

impl ValueType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "null" => Self::Null,
            "boolean" => Self::Boolean,
            "integer" => Self::Integer,
            "number" => Self::Number,
            "string" => Self::String,
            "array" => Self::Array,
            "object" => Self::Object,
            "enum" => Self::Enum,
            "tuple" => Self::Tuple,
            "ref" => Self::Ref,
            unknown_ty => {
                return Err(format!("unknown JSON type `{}`", unknown_ty));
            }
        })
    }
}

/// Retrieves a string property from a JSON value.
pub fn get_prop_str<'a>(value: &'a Value, prop: &str) -> Result<Option<&'a str>, String> {
    match value.get(prop) {
        Some(prop_value) => prop_value
            .as_str()
            .map(Some)
            .ok_or(format!("expected property `{}` to be a string", prop)),
        None => Ok(None),
    }
}

/// Retrieves an array property from a JSON value.
pub fn get_prop_array<'a>(value: &'a Value, prop: &str) -> Result<Option<&'a Vec<Value>>, String> {
    match value.get(prop) {
        Some(prop_value) => prop_value
            .as_array()
            .map(Some)
            .ok_or(format!("expected property `{}` to be an array", prop)),
        None => Ok(None),
    }
}

/// Retrieves an object property from a JSON value.
pub fn get_prop_obj<'a>(
    value: &'a Value,
    prop: &str,
) -> Result<Option<&'a Map<String, Value>>, String> {
    match value.get(prop) {
        Some(prop_value) => prop_value
            .as_object()
            .map(Some)
            .ok_or(format!("expected property `{}` to be an object", prop)),
        None => Ok(None),
    }
}

/// Asserts that a JSON value's type matches as expected.
pub fn assert_value_type(value: &Value, ty: &str) -> Result<(), String> {
    let found_ty = get_prop_str(value, "type")?.ok_or("no type specified".to_owned())?;

    if found_ty == ty {
        Ok(())
    } else {
        Err(format!(
            "mismatched types, expected `{}`, got `{}`",
            ty, found_ty
        ))
    }
}

/// Parses a JSON value's type.
pub fn parse_value_type(value: &Value) -> Result<ValueType, String> {
    ValueType::from_str(match value.get("type") {
        Some(ty) => {
            match ty
                .as_str()
                .ok_or("value type must be a string".to_owned())?
            {
                "array" => {
                    if value.get("prefixItems").is_some() {
                        "tuple"
                    } else {
                        "array"
                    }
                }
                ty_str => ty_str,
            }
        }
        None => None
            .or(value.get("enum").map(|_| "enum"))
            .or(value.get("$ref").map(|_| "ref"))
            .ok_or("value type not specified".to_owned())?,
    })
}

/// Nicely formats a Rust token stream.
pub fn pretty_print_token_stream(tokenstreams: &[TokenStream]) -> String {
    let items = tokenstreams
        .iter()
        .map(|tokens| syn::parse2(tokens.clone()).unwrap())
        .collect();

    let file = syn::File {
        attrs: vec![],
        items,
        shebang: None,
    };

    prettyplease::unparse(&file)
}

/// Takes a JSON property name and returns a valid version of the property,
/// along with the unchanged property name to be used in renaming during
/// serialization.
pub fn renamed_field(name: &str) -> (String, Option<String>) {
    let re = Regex::new("^\\d+").unwrap();
    let renamed_without_leading_digits = re.replace(name, "").to_string();

    let renamed_snake_case = renamed_without_leading_digits.to_case(Case::Snake);

    let renamed_alphanumeric = renamed_snake_case
        .chars()
        .filter_map(|c| (c.is_ascii_alphanumeric() || c == '_').then_some(c))
        .collect::<String>();

    let orig = if renamed_alphanumeric == name {
        None
    } else {
        Some(name.to_owned())
    };

    (renamed_alphanumeric, orig)
}

/// Takes a JSON object name and returns a valid struct name for the object.
pub fn renamed_struct(name: &str) -> String {
    let re = Regex::new("^\\d+").unwrap();
    let renamed_without_leading_digits = re.replace(name, "").to_string();

    let renamed_pascal_case = renamed_without_leading_digits.to_case(Case::Pascal);

    renamed_pascal_case
        .chars()
        .filter_map(|c| (c.is_ascii_alphanumeric()).then_some(c))
        .collect::<String>()
}

/// Takes a JSON object name and returns a valid enum name for the object.
pub fn renamed_enum(name: &str) -> String {
    renamed_struct(name)
}

/// Takes a JSON string from an enum array and returns a valid enum variant
/// name, along with the unchanged property name to be used in renaming during
/// serialization.
pub fn renamed_enum_variant(name: &str) -> (String, Option<String>) {
    let renamed = renamed_struct(name);

    let orig = if renamed == name {
        None
    } else {
        Some(name.to_owned())
    };

    (renamed, orig)
}

/// Takes a JSON ref name and returns a valid type name for the ref.
pub fn renamed_ref(name: &str, root_name: &str) -> String {
    renamed_struct(&format!("{}_def_{}", root_name, name))
}

/// Gets the name of the identifier referenced in a ref field.
pub fn ref_name(path: &[String], root_name: &str) -> String {
    let mut path = path.to_owned();

    if path.is_empty() {
        return root_name.to_owned();
    }

    if let Some(name) = path.get_mut(0) {
        if name == "#" {
            *name = root_name.to_owned();
        }
    }

    if let Some(name) = path.get_mut(1) {
        if name == "$defs" || name == "definitions" {
            *name = "def".to_owned();
        }
    }

    let path_joined = path.join("_");

    renamed_struct(&path_joined)
}

/// Wraps the given type in an `Option` if marked as optional.
pub fn maybe_optional(ty: TokenStream, required: bool) -> TokenStream {
    if required {
        ty
    } else {
        quote!(Option<#ty>)
    }
}

/// Creates a documentation attribute if the given doc string is not empty.
pub fn doc_attribute(maybe_doc: Option<&str>) -> TokenStream {
    match maybe_doc {
        Some(doc_str) => {
            if !doc_str.is_empty() {
                let doc = format!(" {}", doc_str.trim());
                quote!(#[doc = #doc])
            } else {
                quote!()
            }
        }
        None => quote!(),
    }
}

/// Creates a serde rename attribute if the given rename value is not empty.
pub fn rename_attribute(maybe_rename: Option<&str>) -> TokenStream {
    match maybe_rename {
        Some(rename_str) => quote!(#[serde(rename = #rename_str)]),
        None => quote!(),
    }
}
