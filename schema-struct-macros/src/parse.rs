use crate::schema::JsonSchema;
use crate::schema_struct::{SchemaStruct, SchemaStructConfig};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use serde_json::Value;
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

impl Parse for SchemaStructConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut schema_vis = None;
        let mut schema_ident = None;
        let mut schema_def = None;
        let mut schema_validate = None;
        let mut schema_debug = None;

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
                    schema_def = Some(input.parse::<LitBool>()?.value);
                }
                "validate" => {
                    schema_validate = Some(input.parse::<LitBool>()?.value);
                }
                "debug" => {
                    schema_debug = Some(input.parse::<LitBool>()?.value);
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
            validate: schema_validate,
            debug: schema_debug,
            schema: schema_value,
        })
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

/// Parses a JSON schema definition into a Rust struct definition.
pub fn parse_from_schema(input: TokenStream) -> TokenStream {
    let schema_input = input.clone();
    let schema_config = parse_macro_input!(schema_input as SchemaStructConfig);

    let schema = throw_on_err!(SchemaStruct::from_schema(schema_config), input);
    let def = throw_on_err!(schema.to_struct(), input);

    quote!(#def).into()
}
