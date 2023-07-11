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
