#![forbid(unsafe_code)]

/// Internal module from which `serde` and `serde_json` are re-exported. This
/// allows the crate to perform serialization and deserialization operations
/// without requiring users to include `serde` and `serde_json` in their own
/// dependencies.
#[doc(hidden)]
pub mod __internal {
    pub use serde;
    pub use serde_json;
}

pub use schema_struct_macros::schema_struct;
