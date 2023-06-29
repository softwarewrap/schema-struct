use crate::util::*;
use proc_macro2::{Ident, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use syn::Visibility;

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
#[derive(Debug, Clone, Copy)]
pub struct NullField;

/// A boolean field.
#[derive(Debug, Clone, Copy)]
pub struct BooleanField;

/// An integer field.
#[derive(Debug, Clone, Copy)]
pub struct IntegerField;

/// A number field.
#[derive(Debug, Clone, Copy)]
pub struct NumberField;

/// A string field.
#[derive(Debug, Clone, Copy)]
pub struct StringField;

/// An array field.
#[derive(Debug, Clone)]
pub struct ArrayField {
    /// The items in the array.
    pub items: Field,
}

/// An object field.
#[derive(Debug, Clone)]
pub struct ObjectField {
    /// A mapping of the object's field names to values.
    pub fields: HashMap<String, Field>,
}

/// An enum field.
#[derive(Debug, Clone)]
pub struct EnumField {
    /// The names of the enum's variants.
    pub variants: Vec<String>,
}

/// A tuple field.
#[derive(Debug, Clone)]
pub struct TupleField {
    /// The inner tuple fields.
    pub items: Vec<Field>,
}

/// A reference field.
#[derive(Debug, Clone)]
pub struct RefField {
    /// The reference path segments.
    pub path: Vec<String>,
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
pub struct FromSchemaError {
    /// The error message.
    pub message: String,
}

impl Display for FromSchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<&str> for FromSchemaError {
    fn from(value: &str) -> Self {
        Self {
            message: value.to_owned(),
        }
    }
}

impl From<String> for FromSchemaError {
    fn from(value: String) -> Self {
        Self { message: value }
    }
}

/// Parses a JSON schema.
pub trait FromSchema: Sized {
    /// Tries to parse a JSON schema into `Self`.
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, FromSchemaError>;
}

/// Implements `FromSchema` for a primitive JSON type.
macro_rules! impl_from_schema_primitive {
    ( $impl_ty:ty, $json_ty:literal ) => {
        impl FromSchema for $impl_ty {
            fn from_schema(value: &Value, _info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
                assert_value_type(value, $json_ty)?;

                Ok(Self)
            }
        }
    };
}

impl_from_schema_primitive!(NullField, "null");
impl_from_schema_primitive!(BooleanField, "boolean");
impl_from_schema_primitive!(IntegerField, "integer");
impl_from_schema_primitive!(NumberField, "number");
impl_from_schema_primitive!(StringField, "string");

impl FromSchema for ArrayField {
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        assert_value_type(value, "array")?;

        get_prop_obj(value, "items")?.ok_or("array must have property `items`")?;
        let items_value = value
            .get("items")
            .ok_or("array must have property `items`")?;
        let items = Field::from_schema(items_value, info)?;

        Ok(Self { items })
    }
}

impl FromSchema for ObjectField {
    fn from_schema(value: &Value, _info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        assert_value_type(value, "object")?;

        let empty_map = Map::new();
        let empty_vec = Vec::new();

        let properties = get_prop_obj(value, "properties")?.unwrap_or(&empty_map);
        let required_props_array = get_prop_array(value, "required")?.unwrap_or(&empty_vec);
        let mut required_props = HashSet::new();

        for required_prop in required_props_array {
            let prop_name = required_prop
                .as_str()
                .ok_or("required property names must be strings")?;
            required_props.insert(prop_name);
        }

        let mut fields = HashMap::new();

        for (property_name, property_value) in properties {
            let mut property_info = FieldInfo {
                name: property_name.clone(),
                description: None,
                required: required_props.contains(property_name.as_str()),
                subschema: false,
            };
            let parsed_value = Field::from_schema(property_value, &mut property_info)?;
            fields.insert(property_name.to_owned(), parsed_value);
        }

        Ok(Self { fields })
    }
}

impl FromSchema for EnumField {
    fn from_schema(value: &Value, _info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        let variant_values = get_prop_array(value, "enum")?.ok_or("no enum variants specified")?;

        let mut variants = Vec::new();

        for variant in variant_values {
            let variant_name = variant.as_str().ok_or("enum variants must be strings")?;
            variants.push(variant_name.to_owned());
        }

        Ok(Self { variants })
    }
}

impl FromSchema for TupleField {
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        assert_value_type(value, "array")?;

        let tuple_items = get_prop_array(value, "prefixItems")?
            .ok_or("tuple must be defined using the `prefixItems` property")?;

        let mut items = Vec::new();

        for (index, tuple_item) in tuple_items.iter().enumerate() {
            let mut item_info = FieldInfo {
                name: format!("{}{}", info.name, index),
                description: None,
                required: true,
                subschema: false,
            };
            let parsed_value = Field::from_schema(tuple_item, &mut item_info)?;
            items.push(parsed_value);
        }

        Ok(Self { items })
    }
}

impl FromSchema for RefField {
    fn from_schema(value: &Value, _info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        let ref_path = get_prop_str(value, "$ref")?.ok_or("refs must specify `$ref` property")?;
        let path = ref_path.split('/').map(|s| s.to_owned()).collect();

        Ok(Self { path })
    }
}

impl FromSchema for FieldType {
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        Ok(match parse_value_type(value)? {
            ValueType::Null => Self::Null(NullField::from_schema(value, info)?),
            ValueType::Boolean => Self::Boolean(BooleanField::from_schema(value, info)?),
            ValueType::Integer => Self::Integer(IntegerField::from_schema(value, info)?),
            ValueType::Number => Self::Number(NumberField::from_schema(value, info)?),
            ValueType::String => Self::String(StringField::from_schema(value, info)?),
            ValueType::Array => Self::Array(ArrayField::from_schema(value, info)?),
            ValueType::Object => Self::Object(ObjectField::from_schema(value, info)?),
            ValueType::Enum => Self::Enum(EnumField::from_schema(value, info)?),
            ValueType::Tuple => Self::Tuple(TupleField::from_schema(value, info)?),
            ValueType::Ref => Self::Ref(RefField::from_schema(value, info)?),
        })
    }
}

impl FromSchema for Field {
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        let description = get_prop_str(value, "description")?.map(|s| s.to_owned());
        let mut field_info = FieldInfo {
            description,
            ..info.clone()
        };
        let field_ty = FieldType::from_schema(value, &mut field_info)?;

        Ok(Self {
            info: field_info,
            ty: Box::new(field_ty),
        })
    }
}

impl FromSchema for Subschema {
    fn from_schema(value: &Value, info: &mut FieldInfo) -> Result<Self, FromSchemaError> {
        Ok(Self {
            schema: Field::from_schema(value, info)?,
        })
    }
}

/// A definition of a field.
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// The name of the field.
    pub field_name: String,
    /// A different named to be used when serializing and deserializing.
    pub field_rename: Option<String>,
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
pub struct FieldContext {
    /// The name of the root field.
    pub root_name: String,
    /// The name prefix at the current level.
    pub name_prefix: String,
    /// Visibility of the generated items.
    pub vis: Visibility,
    /// The path to the internal module.
    pub internal_path: TokenStream,
}

/// Generates Rust type definitions.
pub trait ToStruct {
    /// Generates Rust type definitions from `Self`.
    fn to_struct(&self, info: &FieldInfo, ctx: &FieldContext) -> FieldDef;
}

/// Implements `ToStruct` for a primitive JSON type.
macro_rules! impl_to_struct_primitive {
    ( $impl_ty:ty, $rust_ty:ty ) => {
        impl ToStruct for $impl_ty {
            fn to_struct(&self, info: &FieldInfo, _ctx: &FieldContext) -> FieldDef {
                let (field_name, field_rename) = renamed_field(&info.name);

                let field_ty = if info.required {
                    quote!($rust_ty)
                } else {
                    quote!(Option<$rust_ty>)
                };

                FieldDef {
                    field_name,
                    field_rename,
                    field_doc: info.description.clone(),
                    field_ty,
                    defs: vec![],
                    defs_doc: vec![],
                }
            }
        }
    };
}

impl_to_struct_primitive!(NullField, ());
impl_to_struct_primitive!(BooleanField, bool);
impl_to_struct_primitive!(IntegerField, i64);
impl_to_struct_primitive!(NumberField, f64);
impl_to_struct_primitive!(StringField, String);

impl ToStruct for ArrayField {
    fn to_struct(&self, info: &FieldInfo, ctx: &FieldContext) -> FieldDef {
        let (field_name, field_rename) = renamed_field(&info.name);
        let inner_field_def = self.items.to_struct(info, ctx);
        let inner_field_ty = &inner_field_def.field_ty;

        let field_ty = if info.required {
            quote!(Vec<#inner_field_ty>)
        } else {
            quote!(Option<Vec<#inner_field_ty>>)
        };

        FieldDef {
            field_name,
            field_rename,
            field_doc: info.description.clone(),
            field_ty,
            defs: inner_field_def.defs,
            defs_doc: inner_field_def.defs_doc,
        }
    }
}

impl ToStruct for ObjectField {
    fn to_struct(&self, info: &FieldInfo, ctx: &FieldContext) -> FieldDef {
        let (field_name, field_rename) = renamed_field(&info.name);
        let struct_name_without_prefix = renamed_struct(&info.name);
        let struct_name = format!("{}{}", ctx.name_prefix, struct_name_without_prefix);
        let struct_ident = format_ident!("{}", struct_name);
        let vis = &ctx.vis;
        let internal_path = &ctx.internal_path;

        let field_ty = if info.required {
            quote!(#struct_ident)
        } else {
            quote!(Option<#struct_ident>)
        };

        let inner_name_prefix = if ctx.name_prefix.is_empty() {
            info.name.clone()
        } else {
            struct_name
        };
        let inner_ctx = FieldContext {
            name_prefix: inner_name_prefix,
            ..ctx.clone()
        };

        let mut defs = Vec::new();
        let mut defs_doc = Vec::new();
        let mut field_tokens = Vec::new();
        let mut field_tokens_doc = Vec::new();

        for inner_field in self.fields.values() {
            let FieldDef {
                field_name: inner_field_name,
                field_rename: inner_field_rename,
                field_doc: inner_field_doc,
                field_ty: inner_field_ty,
                defs: inner_defs,
                defs_doc: inner_defs_doc,
            } = inner_field.to_struct(info, &inner_ctx);

            defs.extend(inner_defs);
            defs_doc.extend(inner_defs_doc);

            let doc_attr = if let Some(doc) = inner_field_doc {
                let doc = format!(" {}", doc);
                quote!(#[doc = #doc])
            } else {
                quote!()
            };

            let renamed_attr = if let Some(renamed) = inner_field_rename {
                quote!(#[serde(rename = #renamed)])
            } else {
                quote!()
            };

            let inner_field_ident = format_ident!("{}", inner_field_name);

            field_tokens.push(quote! {
                #doc_attr
                #renamed_attr
                pub #inner_field_ident: #inner_field_ty,
            });

            field_tokens_doc.push(quote! {
                #doc_attr
                pub #inner_field_ident: #inner_field_ty,
            });
        }

        let doc_attr = if let Some(doc) = &info.description {
            let doc = format!(" {}", doc);
            quote!(#[doc = #doc])
        } else {
            quote!()
        };

        defs.push(quote! {
            #doc_attr
            #[derive(#internal_path::serde::Serialize, #internal_path::serde::Deserialize, Debug, Clone, PartialEq)]
            #vis struct #struct_ident {
                #(#field_tokens)*
            }
        });

        defs.push(quote! {
            impl #struct_ident {
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

        defs_doc.push(quote! {
            #doc_attr
            #vis struct #struct_ident {
                #(#field_tokens_doc)*
            }
        });

        FieldDef {
            field_name,
            field_rename,
            field_doc: info.description.clone(),
            field_ty,
            defs,
            defs_doc,
        }
    }
}

impl ToStruct for EnumField {
    fn to_struct(&self, info: &FieldInfo, ctx: &FieldContext) -> FieldDef {
        let (field_name, field_rename) = renamed_field(&info.name);
        let enum_name_without_prefix = renamed_enum(&info.name);
        let enum_name = format!("{}{}", ctx.name_prefix, enum_name_without_prefix);
        let enum_ident = format_ident!("{}", enum_name);
        let vis = &ctx.vis;
        let internal_path = &ctx.internal_path;

        let field_ty = if info.required {
            quote!(#enum_ident)
        } else {
            quote!(Option<#enum_ident>)
        };

        let mut defs = Vec::new();
        let mut defs_doc = Vec::new();
        let mut variant_tokens = Vec::new();
        let mut variant_tokens_doc = Vec::new();

        for variant in &self.variants {
            let (variant_name, variant_rename) = renamed_enum_variant(variant);
            let variant_ident = format_ident!("{}", variant_name);

            let renamed_attr = if let Some(renamed) = variant_rename {
                quote!(#[serde(rename = #renamed)])
            } else {
                quote!()
            };

            variant_tokens.push(quote! {
                #renamed_attr
                #variant_ident,
            });

            variant_tokens_doc.push(quote! {
                #variant_ident,
            });
        }

        let doc_attr = if let Some(doc) = &info.description {
            let doc = format!(" {}", doc);
            quote!(#[doc = #doc])
        } else {
            quote!()
        };

        defs.push(quote! {
            #doc_attr
            #[derive(#internal_path::serde::Serialize, #internal_path::serde::Deserialize, Debug, Clone, Copy, PartialEq)]
            #vis enum #enum_ident {
                #(#variant_tokens)*
            }
        });

        defs.push(quote! {
            impl #enum_ident {
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

        defs_doc.push(quote! {
            #doc_attr
            #vis enum #enum_ident {
                #(#variant_tokens_doc)*
            }
        });

        FieldDef {
            field_name,
            field_rename,
            field_doc: info.description.clone(),
            field_ty,
            defs,
            defs_doc,
        }
    }
}

impl ToStruct for TupleField {
    fn to_struct(&self, info: &FieldInfo, ctx: &FieldContext) -> FieldDef {
        let (field_name, field_rename) = renamed_field(&info.name);

        let inner_info = FieldInfo {
            required: true,
            ..info.clone()
        };

        let mut defs = Vec::new();
        let mut defs_doc = Vec::new();
        let mut item_tokens = Vec::new();

        for inner_item in &self.items {
            let FieldDef {
                field_name: _inner_item_name,
                field_rename: _inner_item_rename,
                field_doc: _inner_item_doc,
                field_ty: inner_item_ty,
                defs: inner_defs,
                defs_doc: inner_defs_doc,
            } = inner_item.to_struct(&inner_info, ctx);

            defs.extend(inner_defs);
            defs_doc.extend(inner_defs_doc);

            item_tokens.push(quote!(#inner_item_ty));
        }

        let field_ty = if info.required {
            quote!((#(#item_tokens),*))
        } else {
            quote!(Option<(#(#item_tokens),*)>)
        };

        FieldDef {
            field_name,
            field_rename,
            field_doc: info.description.clone(),
            field_ty,
            defs,
            defs_doc,
        }
    }
}

impl ToStruct for RefField {
    fn to_struct(&self, info: &FieldInfo, ctx: &FieldContext) -> FieldDef {
        let (field_name, field_rename) = renamed_field(&info.name);
        let inner_schema_name = ref_name(&self.path, &ctx.root_name);
        let inner_schema_ident = format_ident!("{}", inner_schema_name);

        let field_ty = if info.required {
            quote!(Box<#inner_schema_ident>)
        } else {
            quote!(Option<Box<#inner_schema_ident>>)
        };

        FieldDef {
            field_name,
            field_rename,
            field_doc: info.description.clone(),
            field_ty,
            defs: vec![],
            defs_doc: vec![],
        }
    }
}

impl ToStruct for FieldType {
    fn to_struct(&self, info: &FieldInfo, ctx: &FieldContext) -> FieldDef {
        match self {
            Self::Null(field) => field.to_struct(info, ctx),
            Self::Boolean(field) => field.to_struct(info, ctx),
            Self::Integer(field) => field.to_struct(info, ctx),
            Self::Number(field) => field.to_struct(info, ctx),
            Self::String(field) => field.to_struct(info, ctx),
            Self::Array(field) => field.to_struct(info, ctx),
            Self::Object(field) => field.to_struct(info, ctx),
            Self::Enum(field) => field.to_struct(info, ctx),
            Self::Tuple(field) => field.to_struct(info, ctx),
            Self::Ref(field) => field.to_struct(info, ctx),
        }
    }
}

impl ToStruct for Field {
    fn to_struct(&self, _info: &FieldInfo, ctx: &FieldContext) -> FieldDef {
        let info = if self.info.subschema {
            FieldInfo {
                name: renamed_ref(&self.info.name, &ctx.root_name),
                ..self.info.clone()
            }
        } else {
            self.info.clone()
        };

        self.ty.to_struct(&info, ctx)
    }
}

impl ToStruct for Subschema {
    fn to_struct(&self, info: &FieldInfo, ctx: &FieldContext) -> FieldDef {
        let vis = &ctx.vis;
        let subschema_name = renamed_ref(&info.name, &ctx.root_name);
        let subschema_ident = format_ident!("{}", subschema_name);

        let FieldDef {
            field_name,
            field_rename,
            field_doc,
            field_ty,
            mut defs,
            mut defs_doc,
        } = self.schema.to_struct(info, ctx);

        let doc_attr = if let Some(doc) = &field_doc {
            let doc = format!(" {}", doc);
            quote!(#[doc = #doc])
        } else {
            quote!()
        };

        if defs.is_empty() {
            defs.push(quote! {
                #doc_attr
                #vis type #subschema_ident = #field_ty;
            });
            defs_doc.push(quote! {
                #doc_attr
                #vis type #subschema_ident = #field_ty;
            });
        }

        FieldDef {
            field_name,
            field_rename,
            field_doc,
            field_ty: quote!(#subschema_ident),
            defs,
            defs_doc,
        }
    }
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

        let doc_attr = if let Some(doc) = doc {
            let doc = format!(" {}", doc);
            quote!(#[doc = #doc])
        } else {
            quote!()
        };

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
    pub fn from_schema(config: SchemaStructConfig) -> Result<Self, FromSchemaError> {
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

        let mut subschemas = HashMap::new();

        if let Some(subschema_defs) = subschema_defs {
            for (subschema_name, subschema_value) in subschema_defs {
                let mut subschema_info = FieldInfo {
                    name: subschema_name.clone(),
                    description: None,
                    required: true,
                    subschema: true,
                };
                let subschema = Subschema::from_schema(subschema_value, &mut subschema_info)?;
                subschemas.insert(subschema_name.clone(), subschema);
            }
        }

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
    pub fn to_struct(&self) -> SchemaStructDef {
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
            root_name: self.name.clone(),
            name_prefix: String::new(),
            vis: self.vis.clone(),
            internal_path,
        };

        let mut defs = Vec::new();
        let mut defs_doc = Vec::new();

        for (subschema_name, subschema) in &self.subschemas {
            let subschema_info = FieldInfo {
                name: subschema_name.clone(),
                description: None,
                required: true,
                subschema: true,
            };
            let subschema_def = subschema.to_struct(&subschema_info, &ctx);
            defs.extend(subschema_def.defs);
            defs_doc.extend(subschema_def.defs_doc);
        }

        let root_def = self.root.to_struct(&info, &ctx);
        defs.extend(root_def.defs);
        defs_doc.extend(root_def.defs_doc);

        SchemaStructDef {
            name: self.name.clone(),
            description: self.description.clone(),
            defs,
            defs_doc: self.def.then_some(defs_doc),
        }
    }
}
