//! # Schema Struct
//!
//! Generate Rust struct definitions from JSON schemas at compile-time.
//!
//! ## Example
//!
//! ```
//! use schema_struct::schema_struct;
//!
//! schema_struct!(
//!     schema = {
//!         "$schema": "http://json-schema.org/draft-04/schema#",
//!         "title": "ProductSchema",
//!         "description": "A product from Acme's catalog",
//!         "type": "object",
//!         "properties": {
//!             "id": {
//!                 "description": "The unique identifier for a product",
//!                 "type": "integer"
//!             },
//!             "name": {
//!                 "description": "Name of the product",
//!                 "type": "string"
//!             },
//!             "price": {
//!                 "type": "number",
//!                 "minimum": 0,
//!                 "exclusiveMinimum": true
//!             }
//!         },
//!         "required": ["id", "name", "price"]
//!     }
//! );
//!
//! let product_json = "{\"id\":5,\"name\":\"product name\",\"price\":12.34}";
//! let product = ProductSchema::from_str(product_json).unwrap();
//! assert_eq!(product.id, 5);
//! assert_eq!(product.name, "product name".to_owned());
//! assert_eq!(product.price, 12.34);
//! ```
//!
//! The above example roughly translates to the following definition:
//!
//! ```
//! /// A product from Acme's catalog
//! pub struct ProductSchema {
//!     /// The unique identifier for a product
//!     pub id: i64,
//!     /// Name of the product
//!     pub name: String,
//!     pub price: f64,
//! }
//! ```
//!
//! See the `schema_struct` macro documentation for more information.

#![forbid(unsafe_code)]

mod internal;

/// Internal module from which serialization and deserialization operations
/// are exported. This allows the crate to use `serde` and `serde_json`
/// without requiring users to include them in their own dependencies.
#[doc(hidden)]
pub mod __internal {
    pub use crate::internal::*;
    pub use serde::{Deserialize, Serialize};
}

pub use schema_struct_macros::schema_struct;
