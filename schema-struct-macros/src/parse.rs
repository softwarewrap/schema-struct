use crate::schema::JsonSchema;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, LitBool, LitStr, Token, Visibility};

/// Parses a JSON schema from a string into a `serde_json::Value`.
fn parse_schema_from_str(schema: &str) -> Result<Value, String> {
    match JsonSchema::parse(schema) {
        Ok(_) => serde_json::from_str::<Value>(schema)
            .map_err(|e| format!("error parsing schema as JSON: {}", e)),
        Err(e) => Err(format!("error parsing schema: {:?}", e)),
    }
}

/// Parses a JSON schema that exists in a file.
fn parse_schema_from_file(file: &str) -> Result<Value, String> {
    match fs::read_to_string(file) {
        Ok(value) => parse_schema_from_str(&value),
        Err(e) => Err(e.to_string()),
    }
}

/// Parses a JSON schema that exists at a URL.
fn parse_schema_from_url(url: &str) -> Result<Value, String> {
    match reqwest::blocking::get(url) {
        Ok(res) => match res.text() {
            Ok(value) => parse_schema_from_str(&value),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

/// Configuration of a schema-defined struct.
#[derive(Clone)]
struct SchemaStructConfig {
    /// The visibility level of the struct, e.g. `pub`, `pub(crate)`, or
    /// inherited (private). If not specified or left empty, will default to
    /// inherited.
    vis: Option<Visibility>,
    /// The struct's identifier. If not specified, the schema's `"title"`
    /// property will be used.
    ident: Option<Ident>,
    /// Whether to show the definitions of all generated items in the
    /// top-level struct definition.
    def: bool,
    /// The schema itself, in `serde_json::Value` representation.
    schema: Value,
}

impl Parse for SchemaStructConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut schema_vis = None;
        let mut schema_ident = None;
        let mut schema_def = true;

        let schema_value = loop {
            let keyword = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;

            match keyword.to_string().as_str() {
                "vis" => {
                    schema_vis = Some(input.parse::<Visibility>()?);
                }
                "ident" => {
                    schema_ident = Some(input.parse::<Ident>()?);
                }
                "def" => {
                    schema_def = input.parse::<LitBool>()?.value;
                }
                "schema" => {
                    let schema_tokens = input.parse::<TokenStream2>()?.to_string();
                    break parse_schema_from_str(&schema_tokens)
                        .map_err(|e| syn::Error::new_spanned(schema_tokens, e));
                }
                "file" => {
                    let schema_file = input.parse::<LitStr>()?.value();
                    break parse_schema_from_file(&schema_file)
                        .map_err(|e| syn::Error::new_spanned(schema_file, e));
                }
                "url" => {
                    let schema_url = input.parse::<LitStr>()?.value();
                    break parse_schema_from_url(&schema_url)
                        .map_err(|e| syn::Error::new_spanned(schema_url, e));
                }
                unknown_keyword => {
                    break Err(syn::Error::new_spanned(
                        keyword,
                        format!("unknown keyword '{}'", unknown_keyword),
                    ));
                }
            }

            input.parse::<Token![,]>()?;
        }?;

        Ok(Self {
            vis: schema_vis,
            ident: schema_ident,
            def: schema_def,
            schema: schema_value,
        })
    }
}

/// Retrieves a string property from a JSON value.
fn get_prop_str(value: &Value, prop: &str) -> Result<Option<String>, String> {
    match value.get(prop) {
        Some(prop_value) => prop_value
            .as_str()
            .ok_or(format!("expected property `{}` to be a string", prop))
            .map(|s| Some(s.to_owned())),
        None => Ok(None),
    }
}

/// Nicely formats a Rust token stream.
fn pretty_print_token_stream(tokenstreams: &[TokenStream2]) -> String {
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
fn renamed_field(name: &str) -> (String, Option<String>) {
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
fn renamed_struct(name: &str) -> String {
    let re = Regex::new("^\\d+").unwrap();
    let renamed_without_leading_digits = re.replace(name, "").to_string();

    let renamed_pascal_case = renamed_without_leading_digits.to_case(Case::Pascal);

    renamed_pascal_case
        .chars()
        .filter_map(|c| (c.is_ascii_alphanumeric()).then_some(c))
        .collect::<String>()
}

/// Takes a JSON object name and returns a valid enum name for the object.
fn renamed_enum(name: &str) -> String {
    renamed_struct(name)
}

/// Takes a JSON string from an enum array and returns a valid enum variant name.
fn renamed_enum_variant(name: &str) -> String {
    renamed_struct(name)
}

/// A JSON value type.
enum ValueType {
    Null,
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
    Enum,
    Tuple,
    /// An unrecognized value type.
    Unknown(String),
}

impl ValueType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "null" => Self::Null,
            "boolean" => Self::Boolean,
            "integer" => Self::Integer,
            "number" => Self::Number,
            "string" => Self::String,
            "array" => Self::Array,
            "object" => Self::Object,
            "enum" => Self::Enum,
            "tuple" => Self::Tuple,
            unknown => Self::Unknown(unknown.to_owned()),
        }
    }
}

/// Parses a JSON value's type.
fn parse_value_type(value: &Value) -> Result<ValueType, String> {
    Ok(ValueType::from_str(match value.get("type") {
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
        None => value
            .get("enum")
            .is_some()
            .then_some("enum")
            .ok_or("value type not specified".to_owned())?,
    }))
}

/// A representation of a value parsed from a JSON schema.
#[derive(Debug, Clone)]
struct ParsedValue {
    /// Any types that need to be pre-defined for the value to exist.
    defs: Vec<TokenStream2>,
    /// Any implementations that need to be made.
    impls: Vec<TokenStream2>,
    /// Documentation for the value. This will typically be the value's
    /// `"description"` property.
    doc: String,
    /// A different name to be used for the value when serializing and
    /// deserializing.
    rename: Option<String>,
    /// The value's identifier.
    name: Ident,
    /// The value's type.
    ty: TokenStream2,
}

/// Parses a schema value of type `null`.
fn parse_null(null_value: &Value, name: &str, required: bool) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let name_ident = format_ident!("{}", rust_name);

    let description = get_prop_str(null_value, "description")?;

    let ty = if required {
        quote!(())
    } else {
        quote!(Option<()>)
    };

    Ok(ParsedValue {
        defs: vec![],
        impls: vec![],
        doc: description.unwrap_or_default(),
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of type `boolean`.
fn parse_boolean(boolean_value: &Value, name: &str, required: bool) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let name_ident = format_ident!("{}", rust_name);

    let description = get_prop_str(boolean_value, "description")?;

    let ty = if required {
        quote!(bool)
    } else {
        quote!(Option<bool>)
    };

    Ok(ParsedValue {
        defs: vec![],
        impls: vec![],
        doc: description.unwrap_or_default(),
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of type `integer`.
fn parse_integer(integer_value: &Value, name: &str, required: bool) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let name_ident = format_ident!("{}", rust_name);

    let description = get_prop_str(integer_value, "description")?;

    let ty = if required {
        quote!(i64)
    } else {
        quote!(Option<i64>)
    };

    Ok(ParsedValue {
        defs: vec![],
        impls: vec![],
        doc: description.unwrap_or_default(),
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of type `number`.
fn parse_number(number_value: &Value, name: &str, required: bool) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let name_ident = format_ident!("{}", rust_name);

    let description = get_prop_str(number_value, "description")?;

    let ty = if required {
        quote!(f64)
    } else {
        quote!(Option<f64>)
    };

    Ok(ParsedValue {
        defs: vec![],
        impls: vec![],
        doc: description.unwrap_or_default(),
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of type `string`.
fn parse_string(string_value: &Value, name: &str, required: bool) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let name_ident = format_ident!("{}", rust_name);

    let description = get_prop_str(string_value, "description")?;

    let ty = if required {
        quote!(String)
    } else {
        quote!(Option<String>)
    };

    Ok(ParsedValue {
        defs: vec![],
        impls: vec![],
        doc: description.unwrap_or_default(),
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of type `array`. Because Rust is strongly typed, the
/// type of items in the array must be specified in the schema.
fn parse_array(
    array_value: &Value,
    name: &str,
    name_prefix: &str,
    vis: &Visibility,
    required: bool,
    internal_path: &TokenStream2,
) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let name_ident = format_ident!("{}", rust_name);

    let description = get_prop_str(array_value, "description")?;
    let items_value = array_value
        .get("items")
        .ok_or("array must have property 'items'".to_owned())?;
    let parsed_item_type = parse_value_type(items_value)?;

    let mut defs = vec![];

    let item_type_token = match parsed_item_type {
        ValueType::Null => {
            let inner_null = parse_null(items_value, name, required)?;
            inner_null.ty
        }
        ValueType::Boolean => {
            let inner_boolean = parse_boolean(items_value, name, required)?;
            inner_boolean.ty
        }
        ValueType::Integer => {
            let inner_integer = parse_integer(items_value, name, required)?;
            inner_integer.ty
        }
        ValueType::Number => {
            let inner_number = parse_number(items_value, name, required)?;
            inner_number.ty
        }
        ValueType::String => {
            let inner_string = parse_string(items_value, name, required)?;
            inner_string.ty
        }
        ValueType::Array => {
            let inner_array =
                parse_array(items_value, name, name_prefix, vis, required, internal_path)?;
            defs = inner_array.defs;
            inner_array.ty
        }
        ValueType::Object => {
            let inner_object =
                parse_object(items_value, name, name_prefix, vis, required, internal_path)?;
            defs = inner_object.defs;
            inner_object.ty
        }
        ValueType::Enum => {
            let inner_enum =
                parse_enum(items_value, name, name_prefix, vis, required, internal_path)?;
            defs = inner_enum.defs;
            inner_enum.ty
        }
        ValueType::Tuple => {
            let inner_tuple =
                parse_tuple(items_value, name, name_prefix, vis, required, internal_path)?;
            defs = inner_tuple.defs;
            inner_tuple.ty
        }
        ValueType::Unknown(unknown_type) => {
            return Err(format!("unknown array item type '{}'", unknown_type));
        }
    };

    let ty = if required {
        quote!(Vec<#item_type_token>)
    } else {
        quote!(Option<Vec<#item_type_token>>)
    };

    Ok(ParsedValue {
        defs,
        impls: vec![],
        doc: description.unwrap_or_default(),
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of type `object`.
fn parse_object(
    object_value: &Value,
    name: &str,
    name_prefix: &str,
    vis: &Visibility,
    required: bool,
    internal_path: &TokenStream2,
) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let struct_name = renamed_struct(name);
    let name_ident = format_ident!("{}", rust_name);
    let struct_name_ident = format_ident!("{}{}", name_prefix, struct_name);
    let empty_map = Map::new();

    let description = get_prop_str(object_value, "description")?;
    let properties = object_value
        .get("properties")
        .and_then(|props| props.as_object())
        .unwrap_or(&empty_map);
    let required_props = object_value
        .get("required")
        .and_then(|required| required.as_array())
        .map(|required_array| {
            required_array
                .iter()
                .filter_map(|array_value| array_value.as_str())
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();

    let mut parsed_prop_defs = Vec::new();
    let mut parsed_prop_impls = Vec::new();
    let mut parsed_prop_fields = Vec::new();

    let prop_name_prefix = if name_prefix.is_empty() {
        name.to_owned()
    } else {
        struct_name_ident.to_string()
    };

    for (property_name, property_value) in properties {
        let ParsedValue {
            defs,
            impls,
            doc,
            rename,
            name,
            ty,
        } = parse_value(
            property_value,
            property_name,
            &prop_name_prefix,
            vis,
            required_props.contains(property_name.as_str()),
            internal_path,
        )?;

        let renamed_attr = if let Some(renamed) = rename {
            quote!(#[serde(rename = #renamed)])
        } else {
            quote!()
        };

        parsed_prop_defs.extend(defs);
        parsed_prop_impls.extend(impls);
        parsed_prop_fields.push(quote! {
            #[doc = #doc]
            #renamed_attr
            pub #name: #ty,
        });
    }

    let def_doc = description.unwrap_or_default();

    let mut defs = parsed_prop_defs;
    defs.push(quote! {
        #[doc = #def_doc]
        #[derive(#internal_path::serde::Serialize, #internal_path::serde::Deserialize, Debug, Clone, PartialEq)]
        #vis struct #struct_name_ident {
            #(#parsed_prop_fields)*
        }
    });

    let mut impls = parsed_prop_impls;
    impls.push(quote! {
        impl #struct_name_ident {
            /// Deserializes a JSON string into this type.
            pub fn from_str(json: &str) -> #internal_path::serde_json::Result<Self> {
                #internal_path::serde_json::from_str(json)
            }

            /// Serializes this type into a JSON string.
            pub fn to_str(&self) -> #internal_path::serde_json::Result<String> {
                #internal_path::serde_json::to_string(self)
            }
        }
    });

    let ty = if required {
        quote!(#struct_name_ident)
    } else {
        quote!(Option<#struct_name_ident>)
    };

    Ok(ParsedValue {
        defs,
        impls,
        doc: def_doc,
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of an enum type.
fn parse_enum(
    enum_value: &Value,
    name: &str,
    name_prefix: &str,
    vis: &Visibility,
    required: bool,
    internal_path: &TokenStream2,
) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let enum_name = renamed_enum(name);
    let name_ident = format_ident!("{}", rust_name);
    let enum_name_ident = format_ident!("{}{}", name_prefix, enum_name);

    let description = get_prop_str(enum_value, "description")?;
    let variants = enum_value
        .get("enum")
        .ok_or("no enum variants specified".to_owned())?
        .as_array()
        .ok_or("enum variants must be specified as an array".to_owned())?;

    let mut variant_defs = Vec::new();

    for variant in variants {
        let variant_name = variant
            .as_str()
            .ok_or("enum variants must be strings".to_owned())?;
        let variant_ident = format_ident!("{}", renamed_enum_variant(variant_name));
        variant_defs.push(quote! {
            #[serde(rename = #variant_name)]
            #variant_ident,
        });
    }

    let def_doc = description.unwrap_or_default();

    let defs = vec![quote! {
        #[doc = #def_doc]
        #[derive(#internal_path::serde::Serialize, #internal_path::serde::Deserialize, Debug, Clone, Copy, PartialEq)]
        #vis enum #enum_name_ident {
            #(#variant_defs)*
        }
    }];

    let impls = vec![quote! {
        impl #enum_name_ident {
            /// Deserializes a JSON string into this type.
            pub fn from_str(json: &str) -> #internal_path::serde_json::Result<Self> {
                #internal_path::serde_json::from_str(json)
            }

            /// Serializes this type into a JSON string.
            pub fn to_str(&self) -> #internal_path::serde_json::Result<String> {
                #internal_path::serde_json::to_string(self)
            }
        }
    }];

    let ty = if required {
        quote!(#enum_name_ident)
    } else {
        quote!(Option<#enum_name_ident>)
    };

    Ok(ParsedValue {
        defs,
        impls,
        doc: def_doc,
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of a tuple type.
fn parse_tuple(
    tuple_value: &Value,
    name: &str,
    name_prefix: &str,
    vis: &Visibility,
    required: bool,
    internal_path: &TokenStream2,
) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let name_ident = format_ident!("{}", rust_name);

    let description = get_prop_str(tuple_value, "description")?;
    let tuple_items = tuple_value
        .get("prefixItems")
        .ok_or("tuple must be defined using `prefixItems` property".to_owned())?
        .as_array()
        .ok_or("tuple `prefixItems` must be specified as an array".to_owned())?;

    let mut tuple_types = Vec::new();
    let mut tuple_defs = Vec::new();
    let mut tuple_impls = Vec::new();

    for (index, tuple_item) in tuple_items.iter().enumerate() {
        let tuple_item_name = format!("{}{}", name, index);
        let item = parse_value(
            tuple_item,
            &tuple_item_name,
            name_prefix,
            vis,
            true,
            internal_path,
        )?;
        tuple_types.push(item.ty);
        tuple_defs.extend(item.defs);
        tuple_impls.extend(item.impls);
    }

    let ty = if required {
        quote!((#(#tuple_types),*))
    } else {
        quote!(Option<(#(#tuple_types),*)>)
    };

    Ok(ParsedValue {
        defs: tuple_defs,
        impls: tuple_impls,
        doc: description.unwrap_or_default(),
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value.
fn parse_value(
    value: &Value,
    name: &str,
    name_prefix: &str,
    vis: &Visibility,
    required: bool,
    internal_path: &TokenStream2,
) -> Result<ParsedValue, String> {
    let value_type = parse_value_type(value)?;

    match value_type {
        ValueType::Null => parse_null(value, name, required),
        ValueType::Boolean => parse_boolean(value, name, required),
        ValueType::Integer => parse_integer(value, name, required),
        ValueType::Number => parse_number(value, name, required),
        ValueType::String => parse_string(value, name, required),
        ValueType::Array => parse_array(value, name, name_prefix, vis, required, internal_path),
        ValueType::Object => parse_object(value, name, name_prefix, vis, required, internal_path),
        ValueType::Enum => parse_enum(value, name, name_prefix, vis, required, internal_path),
        ValueType::Tuple => parse_tuple(value, name, name_prefix, vis, required, internal_path),
        ValueType::Unknown(unknown_type) => {
            Err(format!("unknown JSON value type '{}'", unknown_type))
        }
    }
}

/// Helper macro to throw a compiler error on a `Result::Err`.
macro_rules! throw_on_err {
    ( $res:expr, $tokens:expr ) => {
        match $res {
            Ok(value) => value,
            Err(err) => {
                return ::syn::Error::new_spanned(::proc_macro2::TokenStream::from($tokens), err)
                    .to_compile_error()
                    .into();
            }
        }
    };
}

/// Helper macro to throw a compiler error on an `Option::None`.
macro_rules! throw_on_none {
    ( $opt:expr, $err:expr, $tokens:expr ) => {
        match $opt {
            Some(value) => value,
            None => {
                return ::syn::Error::new_spanned(::proc_macro2::TokenStream::from($tokens), $err)
                    .to_compile_error()
                    .into();
            }
        }
    };
}

/// Parses a JSON schema definition into a Rust struct definition.
pub fn parse_from_schema(input: TokenStream) -> TokenStream {
    let schema_input = input.clone();
    let schema_data = parse_macro_input!(schema_input as SchemaStructConfig);
    let SchemaStructConfig {
        vis,
        ident,
        def,
        schema,
    } = schema_data;

    let schema_title =
        throw_on_err!(get_prop_str(&schema, "title"), input).map(|title| renamed_struct(&title));
    let schema_description = throw_on_err!(get_prop_str(&schema, "description"), input)
        .map(|s| format!("{}\n\n", s))
        .unwrap_or_default();
    let schema_title_ident = schema_title.map(|title| format_ident!("{}", title));
    let struct_ident = throw_on_none!(
        ident.or(schema_title_ident),
        "no struct identifier specified in schema or macro invocation",
        input
    );

    let internal_path = match crate_name("schema-struct") {
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident::__internal)
        }
        _ => quote!(::schema_struct::__internal),
    };

    let ParsedValue {
        defs: schema_defs,
        impls: schema_impls,
        ..
    } = throw_on_err!(
        parse_value(
            &schema,
            &struct_ident.to_string(),
            "",
            &vis.unwrap_or(Visibility::Inherited),
            true,
            &internal_path,
        ),
        input
    );

    let schema_doc = if def {
        format!(
            "{}# Full definition\n\n```\n{}\n```",
            schema_description,
            pretty_print_token_stream(&schema_defs)
        )
    } else {
        "".to_owned()
    };

    let (main_def, pre_defs) = schema_defs.split_last().unwrap();

    quote! {
        #(#pre_defs)*

        #[doc = #schema_doc]
        #main_def

        #(#schema_impls)*
    }
    .into()
}
