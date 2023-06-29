#![forbid(unsafe_code)]

mod parse;
mod schema;
mod schema_struct;
mod util;

use crate::parse::parse_from_schema;
use proc_macro::TokenStream;

/// Generates a struct definition from JSON schema.
#[proc_macro]
pub fn schema_struct(input: TokenStream) -> TokenStream {
    parse_from_schema(input)
}
