use super::from_schema::FromSchema;
use super::to_struct::ToStruct;
use super::util::*;
use proc_macro2::{Ident, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Display;
use syn::Visibility;

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
    pub fn from_str(s: &str) -> Result<Self, SchemaStructError> {
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
                return Err(format!("unknown JSON type `{}`", unknown_ty).into());
            }
        })
    }
}

/// Information that applies to all fields.
#[derive(Debug, Clone)]
pub struct FieldInfo {
    /// The name of the field.
    pub name: String,
    /// A description of the field.
    pub description: Option<String>,
    /// Whether the field is required.
    pub required: bool,
    /// Whether the field is a subschema definition.
    pub subschema: bool,
}

/// A null field.
#[derive(Debug, Clone)]
pub struct NullField {
    /// The default value.
    pub default: Option<Value>,
}

/// A boolean field.
#[derive(Debug, Clone)]
pub struct BooleanField {
    /// The default value.
    pub default: Option<Value>,
}

/// An integer field.
#[derive(Debug, Clone)]
pub struct IntegerField {
    /// The default value.
    pub default: Option<Value>,
}

/// A number field.
#[derive(Debug, Clone)]
pub struct NumberField {
    /// The default value.
    pub default: Option<Value>,
}

/// A string field.
#[derive(Debug, Clone)]
pub struct StringField {
    /// The default value.
    pub default: Option<Value>,
}

/// An array field.
#[derive(Debug, Clone)]
pub struct ArrayField {
    /// The items in the array.
    pub items: Field,
    /// The default value.
    pub default: Option<Value>,
}

/// An object field.
#[derive(Debug, Clone)]
pub struct ObjectField {
    /// A mapping of the object's field names to values.
    pub fields: HashMap<String, Field>,
    /// The default value.
    pub default: Option<Value>,
}

/// An enum field.
#[derive(Debug, Clone)]
pub struct EnumField {
    /// The names of the enum's variants.
    pub variants: Vec<String>,
    /// The default value.
    pub default: Option<Value>,
}

/// A tuple field.
#[derive(Debug, Clone)]
pub struct TupleField {
    /// The inner tuple fields.
    pub items: Vec<Field>,
    /// The default value.
    pub default: Option<Value>,
}

/// A reference field.
#[derive(Debug, Clone)]
pub struct RefField {
    /// The reference path segments.
    pub path: Vec<String>,
    /// The default value.
    pub default: Option<Value>,
}

/// The type of a field.
#[derive(Debug, Clone)]
pub enum FieldType {
    Null(NullField),
    Boolean(BooleanField),
    Integer(IntegerField),
    Number(NumberField),
    String(StringField),
    Array(ArrayField),
    Object(ObjectField),
    Enum(EnumField),
    Tuple(TupleField),
    Ref(RefField),
}

/// A field in a schema.
#[derive(Debug, Clone)]
pub struct Field {
    /// Information about the field.
    pub info: FieldInfo,
    /// The field's type info.
    pub ty: Box<FieldType>,
}

/// A subschema within a schema.
#[derive(Debug, Clone)]
pub struct Subschema {
    /// The subschema itself.
    pub schema: Field,
}

/// An error while parsing a schema.
#[derive(Debug, Clone)]
pub struct SchemaStructError {
    /// The error message.
    pub message: String,
}

impl Display for SchemaStructError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SchemaStructError {}

impl From<&str> for SchemaStructError {
    fn from(value: &str) -> Self {
        Self {
            message: value.to_owned(),
        }
    }
}

impl From<String> for SchemaStructError {
    fn from(value: String) -> Self {
        Self { message: value }
    }
}

/// A definition of a field.
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// The name of the field.
    pub field_name: String,
    /// A different named to be used when serializing and deserializing.
    pub field_rename: Option<String>,
    /// The name of a function to use to fill in a default value for the
    /// field. The function itself should be defined in `defs`.
    pub field_default: Option<String>,
    /// Documentation for the field.
    pub field_doc: Option<String>,
    /// The field's type.
    pub field_ty: TokenStream,
    /// All type definitions and implementations associated with the field and
    /// subfields.
    pub defs: Vec<TokenStream>,
    /// Simplified type definitions to be used in documentation.
    pub defs_doc: Vec<TokenStream>,
}

/// Information relating to the context of a field.
#[derive(Clone)]
pub struct FieldContext<'a> {
    /// A reference to the entire schema/struct definition.
    pub schema: &'a SchemaStruct,
    /// The name of the root field.
    pub root_name: String,
    /// The name prefix at the current level.
    pub name_prefix: String,
    /// Visibility of the generated items.
    pub vis: Visibility,
    /// The path to the internal module.
    pub internal_path: TokenStream,
}

/// Configuration of a schema-defined struct.
#[derive(Clone)]
pub struct SchemaStructConfig {
    /// The visibility level of the struct, e.g. `pub`, `pub(crate)`, or
    /// inherited (private). If not specified or left empty, will default to
    /// inherited.
    pub vis: Option<Visibility>,
    /// The struct's identifier. If not specified, the schema's `"title"`
    /// property will be used.
    pub ident: Option<Ident>,
    /// Whether to show the definitions of all generated items in the
    /// top-level struct definition.
    pub def: Option<bool>,
    /// The schema itself, in `serde_json::Value` representation.
    pub schema: Value,
}

/// A definition of a high-level schema struct definition.
#[derive(Debug, Clone)]
pub struct SchemaStructDef {
    /// The data structure name.
    pub name: String,
    /// The data structure description.
    pub description: Option<String>,
    /// All type definitions and implementations associated with the schema.
    pub defs: Vec<TokenStream>,
    /// Simplified type definitions to be used in documentation.
    pub defs_doc: Option<Vec<TokenStream>>,
}

impl ToTokens for SchemaStructDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let doc_description = self
            .description
            .as_ref()
            .map(|s| format!("{}\n\n", s))
            .unwrap_or_default();
        let doc = self
            .defs_doc
            .as_ref()
            .map(|doc| {
                format!(
                    "{}# Full definition\n\n```\n{}\n```",
                    doc_description,
                    pretty_print_token_stream(doc)
                )
            })
            .or(self.description.clone());

        let doc_attr = doc_attribute(doc.as_deref());

        let (main_impl, rest) = self.defs.split_last().unwrap();
        let (main_def, pre_defs) = rest.split_last().unwrap();

        let def = quote! {
            #(#pre_defs)*

            #doc_attr
            #main_def

            #main_impl
        };

        tokens.append_all(def);
    }
}

/// A high-level representation of a schema/struct data structure.
#[derive(Clone)]
pub struct SchemaStruct {
    /// Visibility level of the data structure.
    pub vis: Visibility,
    /// Whether to show the definitions of all generated items in the
    /// top-level data structure definition.
    pub def: bool,
    /// The data structure's identifier name. If not specified, the schema
    /// title will be used.
    pub name: String,
    /// The schema description.
    pub description: Option<String>,
    /// Subschemas defined by the schema.
    pub subschemas: HashMap<String, Subschema>,
    /// The top-level schema object.
    pub root: ObjectField,
}

impl SchemaStruct {
    /// Parses a JSON schema into a representation of a Rust data structure.
    pub fn from_schema(config: SchemaStructConfig) -> Result<Self, SchemaStructError> {
        let SchemaStructConfig {
            vis,
            ident,
            def,
            schema,
        } = config;

        let title = get_prop_str(&schema, "title")?.map(|s| s.to_owned());
        let description = get_prop_str(&schema, "description")?.map(|s| s.to_owned());
        let subschema_defs = None
            .or(get_prop_obj(&schema, "$defs")?)
            .or(get_prop_obj(&schema, "definintions")?);

        let name = ident
            .map(|i| i.to_string())
            .or(title)
            .ok_or("no struct identifier specified in schema or macro invocation")?;

        let subschemas = subschema_defs
            .map(|subschema_defs| {
                subschema_defs
                    .iter()
                    .map(|(subschema_name, subschema_value)| {
                        let mut subschema_info = FieldInfo {
                            name: subschema_name.clone(),
                            description: None,
                            required: true,
                            subschema: true,
                        };
                        Subschema::from_schema(subschema_value, &mut subschema_info)
                            .map(|subschema| (subschema_name.clone(), subschema))
                    })
                    .collect::<Result<HashMap<_, _>, _>>()
            })
            .unwrap_or(Ok(HashMap::new()))?;

        let mut field_info = FieldInfo {
            name: name.clone(),
            description: description.clone(),
            required: true,
            subschema: false,
        };
        let root = ObjectField::from_schema(&schema, &mut field_info)?;

        Ok(Self {
            vis: vis.unwrap_or(Visibility::Inherited),
            name,
            def: def.unwrap_or(true),
            description,
            subschemas,
            root,
        })
    }

    /// Generates Rust code from the data structure representation.
    pub fn to_struct(&self) -> Result<SchemaStructDef, SchemaStructError> {
        let internal_path = match crate_name("schema-struct") {
            Ok(FoundCrate::Name(name)) => {
                let ident = format_ident!("{}", name);
                quote!(::#ident::__internal)
            }
            _ => quote!(::schema_struct::__internal),
        };

        let info = FieldInfo {
            name: self.name.clone(),
            description: self.description.clone(),
            required: true,
            subschema: false,
        };
        let ctx = FieldContext {
            schema: self,
            root_name: self.name.clone(),
            name_prefix: String::new(),
            vis: self.vis.clone(),
            internal_path,
        };

        let (mut defs, mut defs_doc) = self.subschemas.iter().try_fold(
            (Vec::new(), Vec::new()),
            |(mut defs, mut defs_doc), (subschema_name, subschema)| {
                let subschema_info = FieldInfo {
                    name: subschema_name.clone(),
                    description: None,
                    required: true,
                    subschema: true,
                };
                let subschema_def = subschema.to_struct(&subschema_info, &ctx)?;
                defs.extend(subschema_def.defs);
                defs_doc.extend(subschema_def.defs_doc);
                Result::<_, SchemaStructError>::Ok((defs, defs_doc))
            },
        )?;

        let root_def = self.root.to_struct(&info, &ctx)?;
        defs.extend(root_def.defs);
        defs_doc.extend(root_def.defs_doc);

        Ok(SchemaStructDef {
            name: self.name.clone(),
            description: self.description.clone(),
            defs,
            defs_doc: self.def.then_some(defs_doc),
        })
    }
}
