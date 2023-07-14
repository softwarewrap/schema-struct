//! # Schema Struct
//!
//! Procedural macros for the `schema-struct` crate.

#![forbid(unsafe_code)]

mod parse;
mod schema;
mod schema_struct;

use crate::parse::parse_from_schema;
use proc_macro::TokenStream;

/// # Schema Struct
///
/// Generates a struct definition from a JSON schema.
///
/// ## Example
///
/// ```
/// use schema_struct::schema_struct;
///
/// schema_struct!(
///     schema = {
///         "$schema": "http://json-schema.org/draft-04/schema#",
///         "title": "ProductSchema",
///         "description": "A product from Acme's catalog",
///         "type": "object",
///         "properties": {
///             "id": {
///                 "description": "The unique identifier for a product",
///                 "type": "integer"
///             },
///             "name": {
///                 "description": "Name of the product",
///                 "type": "string"
///             },
///             "price": {
///                 "type": "number",
///                 "minimum": 0,
///                 "exclusiveMinimum": true
///             }
///         },
///         "required": ["id", "name", "price"]
///     }
/// );
///
/// let product_json = "{\"id\":5,\"name\":\"product name\",\"price\":12.34}";
/// let product = ProductSchema::from_str(product_json).unwrap();
/// assert_eq!(product.id, 5);
/// assert_eq!(product.name, "product name".to_owned());
/// assert_eq!(product.price, 12.34);
/// ```
///
/// The above example roughly translates to the following definition:
///
/// ```
/// /// A product from Acme's catalog
/// pub struct ProductSchema {
///     /// The unique identifier for a product
///     pub id: i64,
///     /// Name of the product
///     pub name: String,
///     pub price: f64,
/// }
/// ```
///
/// Each generated item also gets `to_str` and `from_str` methods for
/// performing serialization and deserialization.
///
/// Note that the top-level schema value must be an object.
///
/// ## Configuration
///
/// ### Schema
///
/// A schema is always required, and can be provided in one of three ways:
/// directly, via a file, or via a URL. The schema configuration must always
/// be the last option passed to the macro. Any config options after it will
/// be ignored.
///
/// #### Direct schema
///
/// ```ignore
/// schema_struct!(
///     schema = {
///         "$schema": "http://json-schema.org/draft-04/schema#",
///         "type": "object",
///         "properties": {
///             ...
///         }
///     }
/// )
/// ```
///
/// #### Schema from a file
///
/// ```ignore
/// schema_struct!(file = "path/to/schema.json");
/// ```
///
/// #### Schema from a URL
///
/// ```ignore
/// schema_struct!(url = "https://url.where/schema/resides.json");
/// ```
///
/// ### Visibility
///
/// All generated items are private by default, but a visibility level (e.g.
/// `pub`, `pub(crate)`, `pub(super)`, etc.) can be specified with the `vis`
/// option.
///
/// ```ignore
/// schema_struct!(
///     vis = pub,
///     schema = { ... }
/// );
/// ```
///
/// ### Struct identifier
///
/// A custom struct identifier can be provided via the `ident` option. If not
/// specified, the identifier will default to the schema's `"title"` property.
///
/// ```ignore
/// schema_struct!(
///     ident = MyStruct,
///     schema = { ... }
/// );
/// ```
///
/// Note that if neither a custom identifier nor the `"title"` prop are
/// available, an error will be raised.
///
/// ### Type definition documentation
///
/// By default, the generated type definitions will be appended to the doc
/// comment on the top-level struct. This behavior can be disabled with the
/// `def` option.
///
/// ```ignore
/// schema_struct!(
///     def = false,
///     schema = { ... }
/// );
/// ```
///
/// Note that the generated types shown in the doc comment do not represent
/// the full and complete set of generated code, but rather a simpler, more
/// readable representation of the code.
///
/// ### Schema validation
///
/// JSON objects are not validated against the schema when deserializing. The
/// reason for this is that the macro is aimed more at performing compile-time
/// validation via type-level guarantees. That said, runtime schema validation
/// can be enabled via the `validate` option.
///
/// ```ignore
/// schema_struct!(
///     validate = true,
///     schema = { ... }
/// );
/// ```
///
/// ### Debug information
///
/// Currently, the only useful debug information the macro can provide is the
/// full code generated. This includes struct and enum definitions and their
/// implementations, as well as type aliases for references and function
/// definitions for default values. It can be enabled with the `debug` option.
/// When enabled, all generated code will be dumped to stdout.
///
/// ```ignore
/// schema_struct!(
///     debug = true,
///     schema = { ... }
/// );
/// ```
///
/// ## Supported data types
///
/// ### Null
///
/// JSON values of type `null` are supported, and they are represented as the
/// unit type `()`.
///
/// ```ignore
/// { "type": "null" }
/// ```
///
/// ### Boolean
///
/// JSON booleans correspond to the `bool` type.
///
/// ```ignore
/// { "type": "boolean" }
/// ```
///
/// ### Integer
///
/// JSON integers are represented as `i64`s.
///
/// ```ignore
/// { "type": "integer" }
/// ```
///
/// ### Number
///
/// JSON numbers are represented as `f64`s.
///
/// ```ignore
/// { "type": "number" }
/// ```
///
/// ### String
///
/// Strings in JSON correspond to Rust's owned `String`s.
///
/// ```ignore
/// { "type": "string" }
/// ```
///
/// ### Array
///
/// Arrays translate to `Vec`s in Rust. Because of this, arrays are
/// limited to one type of element, and that type must be specified in the
/// schema definition.
///
/// ```ignore
/// {
///     "type": "array",
///     "items": {
///         "type": "integer"
///     }
/// }
/// ```
///
/// The example above would be transformed into a `Vec<i64>`.
///
/// ### Object
///
/// Objects are transformed into struct definitions. Struct names and fields
/// may be renamed to match Rust's naming conventions, but they will still
/// serialize correctly according to the provided schema.
///
/// ```ignore
/// {
///     "myObject": {
///         "type": "object",
///         "properties": {
///             "myProp": {
///                 "type": "integer"
///             }
///         },
///         "required": ["myProp"]
///     }
/// }
/// ```
///
/// The example above would be transformed into:
///
/// ```
/// struct MyObject {
///     my_prop: i64,
/// }
/// ```
///
/// ### Enum
///
/// Enums in JSON schemas are represented as one of an arbitrary number of
/// strings. Each string will become a variant in a Rust enum. Like with
/// objects, an enum's name and fields may be changed to match naming
/// conventions.
///
/// ```ignore
/// {
///     "my_enum": {
///         "enum": ["first_variant", "second_variant", "third_variant"]
///     }
/// }
/// ```
///
/// The example above would be transformed into:
///
/// ```
/// enum MyEnum {
///     FirstVariant,
///     SecondVariant,
///     ThirdVariant,
/// }
/// ```
///
/// ### Tuple
///
/// JSON schemas represent tuples as an array of JSON values. This corresponds
/// nicely to Rust's tuples.
///
/// ```ignore
/// {
///     "type": "array",
///     "prefixItems": [
///         {
///             "type": "integer"
///         },
///         {
///             "type": "string"
///         },
///         {
///             "type": "string"
///         }
///     ]
/// }
/// ```
///
/// The example above would be transformed into the following tuple type:
///
/// ```ignore
/// (i64, String, String)
/// ```
///
/// ### Ref
///
/// References are a very useful feature of JSON schemas. They are supported
/// through the `Box` smart pointer, in order to allow potentially
/// self-referential data structures. All refs must point to either the root
/// object itself or a defined subschema.
///
/// A ref to the root object:
///
/// ```ignore
/// {
///     "$schema": "...",
///     "title": "SchemaWithRef",
///     "type": "object",
///     "properties": {
///         "self_referential_field": {
///             "$ref": "#"
///         }
///     }
/// }
/// ```
///
/// would translate into:
///
/// ```
/// struct SchemaWithRef {
///     self_referential_field: Option<Box<SchemaWithRef>>,
/// }
/// ```
///
/// A ref to a subschema:
///
/// ```ignore
/// {
///     "$schema": "...",
///     "title": "SchemaWithRef",
///     "type": "object",
///     "$defs": {
///         "myInteger": {
///             "type": "integer"
///         }
///     },
///     "properties": {
///         "my_integer_field": {
///             "$ref": "#/$defs/myInteger"
///         }
///     }
/// }
/// ```
///
/// would translate into:
///
/// ```
/// pub type SchemaWithRefDefMyInteger = i64;
///
/// struct SchemaWithRef {
///     my_integer_field: Option<Box<SchemaWithRefDefMyInteger>>,
/// }
/// ```
///
/// In this example, a type alias is generated for the inner integer type. For
/// non-primitive subschema types, full type definitions will be generated
/// instead.
///
/// ## Optional fields
///
/// By default, JSON schemas assume that all fields are optional. To mark a
/// field as required, use the `"required"` property. Any fields not labeled
/// as required will have their types wrapped in an `Option`.
///
/// ```ignore
/// {
///     "$schema": "...",
///     "title": "Person",
///     "type": "object",
///     "properties": {
///         "name": {
///             "type": "string"
///         },
///         "age": {
///             "type": "number"
///         }
///     },
///     "required": ["name"]
/// }
/// ```
///
/// This will become:
///
/// ```
/// struct Person {
///     name: String,
///     age: Option<i64>,
/// }
/// ```
///
/// ## Default values
///
/// Default values can be provided for JSON values of any type. If a field is
/// omitted when deserializing, the default value provided will be used.
///
/// ```ignore
/// {
///     "type": "object",
///     "properties": {
///         "null_prop": {
///             "type": "null",
///             "default": null
///         },
///         "boolean_prop": {
///             "type": "boolean",
///             "default": true
///         },
///         "integer_prop": {
///             "type": "integer",
///             "default": 7
///         },
///         "number_prop": {
///             "type": "number",
///             "default": 3.45
///         },
///         "string_prop": {
///             "type": "string",
///             "default": "Hello, world!"
///         },
///         "array_field": {
///             "type": "array",
///             "items": {
///                 "type": "integer"
///             },
///             "default": [7, 8, 9]
///         },
///         "object_field": {
///             "type": "object",
///             "properties": {
///                 "inner_field": {
///                     "type": "string"
///                 }
///             },
///             "required": ["inner_field"],
///             "default": {
///                 "inner_field": "an inner object field"
///             }
///         },
///         "enum_field": {
///             "enum": ["first", "second", "third"],
///             "default": "first"
///         },
///         "tuple_field": {
///             "type": "array",
///             "prefixItems": [
///                 {
///                     "type": "integer"
///                 },
///                 {
///                     "type": "string"
///                 },
///                 {
///                     "type": "string"
///                 }
///             ],
///             "default": [1600, "Pennsylvania", "Avenue"]
///         }
///     }
/// }
/// ```
///
/// ### Default propagation
///
/// When a property is omitted in the declaration of a default property value,
/// the default value of the inner property is used. If the inner property
/// does not define a default value, then `null` will be used instead. If the
/// property is not nullable, an error will be raised.
///
/// ## Documentation
///
/// Struct definitions and fields on them can be documented using the
/// "description" property. Attach a description to any value, including the
/// top-level schema definintion and it will be included as a doc comment in
/// or on the generated data structure.
#[proc_macro]
pub fn schema_struct(input: TokenStream) -> TokenStream {
    parse_from_schema(input)
}
