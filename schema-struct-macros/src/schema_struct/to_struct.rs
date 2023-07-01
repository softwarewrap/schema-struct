use super::types::*;
use super::util::*;
use quote::{format_ident, quote};

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
                let field_ty = maybe_optional(quote!($rust_ty), info.required);

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
        let field_ty = maybe_optional(quote!(Vec<#inner_field_ty>), info.required);

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
        let field_ty = maybe_optional(quote!(#struct_ident), info.required);

        let inner_name_prefix = if ctx.name_prefix.is_empty() {
            info.name.clone()
        } else {
            struct_name
        };
        let inner_ctx = FieldContext {
            name_prefix: inner_name_prefix,
            ..ctx.clone()
        };

        let (mut defs, mut defs_doc, field_tokens, field_tokens_doc) = self.fields.values().fold(
            (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            |(mut defs, mut defs_doc, mut field_tokens, mut field_tokens_doc), inner_field| {
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

                let doc_attr = doc_attribute(inner_field_doc.as_deref());

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

                (defs, defs_doc, field_tokens, field_tokens_doc)
            },
        );

        let doc_attr = doc_attribute(info.description.as_deref());

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
        let field_ty = maybe_optional(quote!(#enum_ident), info.required);

        let (variant_tokens, variant_tokens_doc) = self.variants.iter().fold(
            (Vec::new(), Vec::new()),
            |(mut variant_tokens, mut variant_tokens_doc), variant| {
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

                (variant_tokens, variant_tokens_doc)
            },
        );

        let doc_attr = doc_attribute(info.description.as_deref());

        let mut defs = Vec::new();
        let mut defs_doc = Vec::new();

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

        let (defs, defs_doc, item_tokens) = self.items.iter().fold(
            (Vec::new(), Vec::new(), Vec::new()),
            |(mut defs, mut defs_doc, mut item_tokens), inner_item| {
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

                (defs, defs_doc, item_tokens)
            },
        );

        let field_ty = maybe_optional(quote!((#(#item_tokens),*)), info.required);

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
        let field_ty = maybe_optional(quote!(Box<#inner_schema_ident>), info.required);

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

        let doc_attr = doc_attribute(field_doc.as_deref());

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
