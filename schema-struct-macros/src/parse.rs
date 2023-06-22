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
fn get_prop_str(value: &Value, prop: &str) -> Option<String> {
    value.as_object().and_then(|obj| {
        obj.get(prop)
            .and_then(|title| title.as_str())
            .map(|s| s.to_owned())
    })
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
fn parse_null(null: &Value, name: &str, required: bool) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let name_ident = format_ident!("{}", rust_name);

    let description = null
        .get("description")
        .and_then(|description| description.as_str());

    let ty = if required {
        quote!(())
    } else {
        quote!(Option<()>)
    };

    Ok(ParsedValue {
        defs: vec![],
        impls: vec![],
        doc: description.unwrap_or_default().to_owned(),
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of type `boolean`.
fn parse_boolean(boolean: &Value, name: &str, required: bool) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let name_ident = format_ident!("{}", rust_name);

    let description = boolean
        .get("description")
        .and_then(|description| description.as_str());

    let ty = if required {
        quote!(bool)
    } else {
        quote!(Option<bool>)
    };

    Ok(ParsedValue {
        defs: vec![],
        impls: vec![],
        doc: description.unwrap_or_default().to_owned(),
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of type `integer`.
fn parse_integer(integer: &Value, name: &str, required: bool) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let name_ident = format_ident!("{}", rust_name);

    let description = integer
        .get("description")
        .and_then(|description| description.as_str());

    let ty = if required {
        quote!(i64)
    } else {
        quote!(Option<i64>)
    };

    Ok(ParsedValue {
        defs: vec![],
        impls: vec![],
        doc: description.unwrap_or_default().to_owned(),
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of type `number`.
fn parse_number(number: &Value, name: &str, required: bool) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let name_ident = format_ident!("{}", rust_name);

    let description = number
        .get("description")
        .and_then(|description| description.as_str());

    let ty = if required {
        quote!(f64)
    } else {
        quote!(Option<f64>)
    };

    Ok(ParsedValue {
        defs: vec![],
        impls: vec![],
        doc: description.unwrap_or_default().to_owned(),
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of type `string`.
fn parse_string(string: &Value, name: &str, required: bool) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let name_ident = format_ident!("{}", rust_name);

    let description = string
        .get("description")
        .and_then(|description| description.as_str());

    let ty = if required {
        quote!(String)
    } else {
        quote!(Option<String>)
    };

    Ok(ParsedValue {
        defs: vec![],
        impls: vec![],
        doc: description.unwrap_or_default().to_owned(),
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of type `array`. Because Rust is strongly typed, the
/// type of items in the array must be specified in the schema.
fn parse_array(
    array: &Value,
    name: &str,
    name_prefix: &str,
    vis: &Visibility,
    required: bool,
    internal_path: &TokenStream2,
) -> Result<ParsedValue, String> {
    let (rust_name, json_name) = renamed_field(name);
    let name_ident = format_ident!("{}", rust_name);
    let empty_map = Map::new();

    let description = array
        .get("description")
        .and_then(|description| description.as_str());
    let items_value = array
        .get("items")
        .ok_or("array must have property 'items'".to_owned())?;
    let items = items_value.as_object().unwrap_or(&empty_map);
    let item_type = items
        .get("type")
        .ok_or("array item type not specified".to_owned())?
        .as_str()
        .ok_or("array item type must be a string".to_owned())?;

    let mut defs = vec![];

    let item_type_token = match item_type {
        "null" => {
            let inner_null = parse_null(items_value, name, required)?;
            inner_null.ty
        }
        "boolean" => {
            let inner_boolean = parse_boolean(items_value, name, required)?;
            inner_boolean.ty
        }
        "integer" => {
            let inner_integer = parse_integer(items_value, name, required)?;
            inner_integer.ty
        }
        "number" => {
            let inner_number = parse_number(items_value, name, required)?;
            inner_number.ty
        }
        "string" => {
            let inner_string = parse_string(items_value, name, required)?;
            inner_string.ty
        }
        "array" => {
            let inner_array =
                parse_array(items_value, name, name_prefix, vis, required, internal_path)?;
            defs = inner_array.defs;
            inner_array.ty
        }
        "object" => {
            let inner_object =
                parse_object(items_value, name, name_prefix, vis, required, internal_path)?;
            defs = inner_object.defs;
            inner_object.ty
        }
        unknown_type => {
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
        doc: description.unwrap_or_default().to_owned(),
        rename: json_name,
        name: name_ident,
        ty,
    })
}

/// Parses a schema value of type `object`.
fn parse_object(
    object: &Value,
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

    let description = object
        .get("description")
        .and_then(|description| description.as_str());
    let properties = object
        .get("properties")
        .and_then(|props| props.as_object())
        .unwrap_or(&empty_map);
    let required_props = object
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

    let def_doc = description.unwrap_or_default().to_owned();

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

/// Parses a schema value.
fn parse_value(
    value: &Value,
    name: &str,
    name_prefix: &str,
    vis: &Visibility,
    required: bool,
    internal_path: &TokenStream2,
) -> Result<ParsedValue, String> {
    let value_type = value
        .get("type")
        .ok_or("value type not specified".to_owned())?
        .as_str()
        .ok_or("value type must be a string".to_owned())?;

    match value_type {
        "null" => parse_null(value, name, required),
        "boolean" => parse_boolean(value, name, required),
        "integer" => parse_integer(value, name, required),
        "number" => parse_number(value, name, required),
        "string" => parse_string(value, name, required),
        "array" => parse_array(value, name, name_prefix, vis, required, internal_path),
        "object" => parse_object(value, name, name_prefix, vis, required, internal_path),
        unknown_type => Err(format!("unknown JSON value type '{}'", unknown_type)),
    }
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

    let schema_title = get_prop_str(&schema, "title").map(|title| renamed_struct(&title));
    let schema_description = get_prop_str(&schema, "description")
        .map(|s| format!("{}\n\n", s))
        .unwrap_or_default();
    let schema_title_ident = schema_title.map(|title| format_ident!("{}", title));
    let struct_ident = match ident.or(schema_title_ident) {
        Some(struct_ident) => struct_ident,
        None => {
            return syn::Error::new_spanned(
                TokenStream2::from(input),
                "no struct identifier specified in schema or macro invocation",
            )
            .to_compile_error()
            .into();
        }
    };

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
    } = match parse_value(
        &schema,
        &struct_ident.to_string(),
        "",
        &vis.unwrap_or(Visibility::Inherited),
        true,
        &internal_path,
    ) {
        Ok(value) => value,
        Err(e) => {
            return syn::Error::new_spanned(TokenStream2::from(input), e)
                .to_compile_error()
                .into();
        }
    };

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
