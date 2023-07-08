use super::types::*;
use super::util::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde_json::Value;

/// Generates Rust type definitions.
pub trait ToStruct {
    /// Generates Rust type definitions from `Self`.
    fn to_struct(
        &self,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError>;

    /// Generates a token stream representing the default value for `Self`.
    fn parse_default(
        &self,
        value: Option<&Value>,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError>;
}

impl ToStruct for NullField {
    fn to_struct(
        &self,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError> {
        let (field_name, field_rename) = renamed_field(&info.name);
        let field_ty = maybe_optional(quote!(()), info.required);
        let mut defs = Vec::new();

        let field_default =
            self.parse_default(self.default.as_ref(), info, ctx)?
                .map(|default_value| {
                    let field_default = default_fn_name(&ctx.name_prefix, &info.name);
                    let field_default_ident = format_ident!("{}", field_default);

                    defs.push(if info.required {
                        quote! {
                            fn #field_default_ident() {}
                        }
                    } else {
                        quote! {
                            fn #field_default_ident() -> Option<()> {
                                #default_value
                            }
                        }
                    });

                    field_default
                });

        Ok(FieldDef {
            field_name,
            field_rename,
            field_default,
            field_doc: info.description.clone(),
            field_ty,
            defs,
            defs_doc: Vec::new(),
        })
    }

    fn parse_default(
        &self,
        value: Option<&Value>,
        info: &FieldInfo,
        _ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError> {
        value
            .map(|default| {
                default
                    .as_null()
                    .ok_or("expected default value to be null".into())
                    .map(|_val| maybe_optional_value(quote!(()), info.required))
            })
            .invert()
    }
}

impl ToStruct for BooleanField {
    fn to_struct(
        &self,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError> {
        let (field_name, field_rename) = renamed_field(&info.name);
        let field_ty = maybe_optional(quote!(bool), info.required);
        let mut defs = Vec::new();

        let field_default =
            self.parse_default(self.default.as_ref(), info, ctx)?
                .map(|default_value| {
                    let field_default = default_fn_name(&ctx.name_prefix, &info.name);
                    let field_default_ident = format_ident!("{}", field_default);
                    let fn_return = maybe_optional(quote!(bool), info.required);

                    defs.push(quote! {
                        fn #field_default_ident() -> #fn_return {
                            #default_value
                        }
                    });

                    field_default
                });

        Ok(FieldDef {
            field_name,
            field_rename,
            field_default,
            field_doc: info.description.clone(),
            field_ty,
            defs,
            defs_doc: Vec::new(),
        })
    }

    fn parse_default(
        &self,
        value: Option<&Value>,
        info: &FieldInfo,
        _ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError> {
        value
            .map(|default| {
                default
                    .as_bool()
                    .ok_or("expected default value to be a boolean".into())
                    .map(|val| maybe_optional_value(quote!(#val), info.required))
            })
            .invert()
    }
}

impl ToStruct for IntegerField {
    fn to_struct(
        &self,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError> {
        let (field_name, field_rename) = renamed_field(&info.name);
        let field_ty = maybe_optional(quote!(i64), info.required);
        let mut defs = Vec::new();

        let field_default =
            self.parse_default(self.default.as_ref(), info, ctx)?
                .map(|default_value| {
                    let field_default = default_fn_name(&ctx.name_prefix, &info.name);
                    let field_default_ident = format_ident!("{}", field_default);
                    let fn_return = maybe_optional(quote!(i64), info.required);

                    defs.push(quote! {
                        fn #field_default_ident() -> #fn_return {
                            #default_value
                        }
                    });

                    field_default
                });

        Ok(FieldDef {
            field_name,
            field_rename,
            field_default,
            field_doc: info.description.clone(),
            field_ty,
            defs,
            defs_doc: Vec::new(),
        })
    }

    fn parse_default(
        &self,
        value: Option<&Value>,
        info: &FieldInfo,
        _ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError> {
        value
            .map(|default| {
                default
                    .as_i64()
                    .ok_or("expected default value to be an integer".into())
                    .map(|val| maybe_optional_value(quote!(#val), info.required))
            })
            .invert()
    }
}

impl ToStruct for NumberField {
    fn to_struct(
        &self,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError> {
        let (field_name, field_rename) = renamed_field(&info.name);
        let field_ty = maybe_optional(quote!(f64), info.required);
        let mut defs = Vec::new();

        let field_default =
            self.parse_default(self.default.as_ref(), info, ctx)?
                .map(|default_value| {
                    let field_default = default_fn_name(&ctx.name_prefix, &info.name);
                    let field_default_ident = format_ident!("{}", field_default);
                    let fn_return = maybe_optional(quote!(f64), info.required);

                    defs.push(quote! {
                        fn #field_default_ident() -> #fn_return {
                            #default_value
                        }
                    });

                    field_default
                });

        Ok(FieldDef {
            field_name,
            field_rename,
            field_default,
            field_doc: info.description.clone(),
            field_ty,
            defs,
            defs_doc: Vec::new(),
        })
    }

    fn parse_default(
        &self,
        value: Option<&Value>,
        info: &FieldInfo,
        _ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError> {
        value
            .map(|default| {
                default
                    .as_f64()
                    .ok_or("expected default value to be a number".into())
                    .map(|val| maybe_optional_value(quote!(#val), info.required))
            })
            .invert()
    }
}

impl ToStruct for StringField {
    fn to_struct(
        &self,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError> {
        let (field_name, field_rename) = renamed_field(&info.name);
        let field_ty = maybe_optional(quote!(String), info.required);
        let mut defs = Vec::new();

        let field_default =
            self.parse_default(self.default.as_ref(), info, ctx)?
                .map(|default_value| {
                    let field_default = default_fn_name(&ctx.name_prefix, &info.name);
                    let field_default_ident = format_ident!("{}", field_default);
                    let fn_return = maybe_optional(quote!(String), info.required);

                    defs.push(quote! {
                        fn #field_default_ident() -> #fn_return {
                            #default_value
                        }
                    });

                    field_default
                });

        Ok(FieldDef {
            field_name,
            field_rename,
            field_default,
            field_doc: info.description.clone(),
            field_ty,
            defs,
            defs_doc: Vec::new(),
        })
    }

    fn parse_default(
        &self,
        value: Option<&Value>,
        info: &FieldInfo,
        _ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError> {
        value
            .map(|default| {
                default
                    .as_str()
                    .ok_or("expected default value to be a string".into())
                    .map(|val| maybe_optional_value(quote!(#val.to_owned()), info.required))
            })
            .invert()
    }
}

impl ToStruct for ArrayField {
    fn to_struct(
        &self,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError> {
        let (field_name, field_rename) = renamed_field(&info.name);

        let inner_name_prefix = format!("{}Items", ctx.name_prefix);
        let inner_ctx = FieldContext {
            name_prefix: inner_name_prefix,
            ..ctx.clone()
        };

        let inner_field_def = self.items.to_struct(info, &inner_ctx)?;
        let inner_field_ty = &inner_field_def.field_ty;
        let field_ty = maybe_optional(quote!(Vec<#inner_field_ty>), info.required);
        let mut defs = inner_field_def.defs;

        let field_default =
            self.parse_default(self.default.as_ref(), info, ctx)?
                .map(|default_value| {
                    let field_default = default_fn_name(&ctx.name_prefix, &info.name);
                    let field_default_ident = format_ident!("{}", field_default);
                    let fn_return = maybe_optional(quote!(Vec<#inner_field_ty>), info.required);

                    defs.push(quote! {
                        fn #field_default_ident() -> #fn_return {
                            #default_value
                        }
                    });

                    field_default
                });

        Ok(FieldDef {
            field_name,
            field_rename,
            field_default,
            field_doc: info.description.clone(),
            field_ty,
            defs,
            defs_doc: inner_field_def.defs_doc,
        })
    }

    fn parse_default(
        &self,
        value: Option<&Value>,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError> {
        let inner_name_prefix = format!("{}Items", ctx.name_prefix);
        let inner_ctx = FieldContext {
            name_prefix: inner_name_prefix,
            ..ctx.clone()
        };

        value
            .map(|default| {
                default
                    .as_array()
                    .ok_or("expected default value to be an array".into())
                    .and_then(|values| {
                        values
                            .iter()
                            .map(|val| self.items.parse_default(Some(val), info, &inner_ctx))
                            .collect::<Result<Vec<_>, _>>()
                            .map(|defaults| {
                                let defaults = defaults
                                    .iter()
                                    .map(|default| default.clone().unwrap_or(quote!(None)))
                                    .collect::<Vec<_>>();

                                maybe_optional_value(quote!(vec![#(#defaults),*]), info.required)
                            })
                    })
            })
            .invert()
    }
}

impl ToStruct for ObjectField {
    fn to_struct(
        &self,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError> {
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

        let (mut defs, mut defs_doc, field_tokens, field_tokens_doc) =
            self.fields.values().try_fold(
                (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                |(mut defs, mut defs_doc, mut field_tokens, mut field_tokens_doc), inner_field| {
                    let FieldDef {
                        field_name: inner_field_name,
                        field_rename: inner_field_rename,
                        field_default: inner_field_default,
                        field_doc: inner_field_doc,
                        field_ty: inner_field_ty,
                        defs: inner_defs,
                        defs_doc: inner_defs_doc,
                    } = inner_field.to_struct(info, &inner_ctx)?;

                    defs.extend(inner_defs);
                    defs_doc.extend(inner_defs_doc);

                    let doc_attr = doc_attribute(inner_field_doc.as_deref());
                    let renamed_attr = rename_attribute(inner_field_rename.as_deref());
                    let default_attr = default_attribute(inner_field_default.as_deref());

                    let inner_field_ident = format_ident!("{}", inner_field_name);

                    field_tokens.push(quote! {
                        #doc_attr
                        #renamed_attr
                        #default_attr
                        pub #inner_field_ident: #inner_field_ty,
                    });

                    field_tokens_doc.push(quote! {
                        #doc_attr
                        pub #inner_field_ident: #inner_field_ty,
                    });

                    Result::<_, SchemaStructError>::Ok((
                        defs,
                        defs_doc,
                        field_tokens,
                        field_tokens_doc,
                    ))
                },
            )?;

        let field_default =
            self.parse_default(self.default.as_ref(), info, ctx)?
                .map(|default_value| {
                    let field_default = default_fn_name(&ctx.name_prefix, &info.name);
                    let field_default_ident = format_ident!("{}", field_default);
                    let fn_return = maybe_optional(quote!(#struct_ident), info.required);

                    defs.push(quote! {
                        fn #field_default_ident() -> #fn_return {
                            #default_value
                        }
                    });

                    field_default
                });

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

        Ok(FieldDef {
            field_name,
            field_rename,
            field_default,
            field_doc: info.description.clone(),
            field_ty,
            defs,
            defs_doc,
        })
    }

    fn parse_default(
        &self,
        value: Option<&Value>,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError> {
        let struct_name_without_prefix = renamed_struct(&info.name);
        let struct_name = format!("{}{}", ctx.name_prefix, struct_name_without_prefix);
        let struct_ident = format_ident!("{}", struct_name);

        let inner_name_prefix = if ctx.name_prefix.is_empty() {
            info.name.clone()
        } else {
            struct_name
        };
        let inner_ctx = FieldContext {
            name_prefix: inner_name_prefix,
            ..ctx.clone()
        };

        value
            .map(|default| {
                default
                    .as_object()
                    .ok_or("expected default value to be an object".into())
                    .and_then(|values| {
                        self.fields
                            .iter()
                            .map(|(field_name, field)| {
                                let (renamed_field_name, _) = renamed_field(field_name);

                                match values.get(field_name) {
                                    Some(field_value) => field
                                        .parse_default(Some(field_value), info, &inner_ctx)
                                        .map(|inner| inner.unwrap_or(quote!(None))),
                                    None => Ok(quote!(None)),
                                }
                                .map(|value_tokens| {
                                    let field_ident = format_ident!("{}", renamed_field_name);
                                    quote!(#field_ident: #value_tokens,)
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()
                            .map(|defaults| {
                                maybe_optional_value(
                                    quote! {
                                        #struct_ident {
                                            #(#defaults)*
                                        }
                                    },
                                    info.required,
                                )
                            })
                    })
            })
            .invert()
    }
}

impl ToStruct for EnumField {
    fn to_struct(
        &self,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError> {
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

                let renamed_attr = rename_attribute(variant_rename.as_deref());

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

        let mut defs = Vec::new();
        let mut defs_doc = Vec::new();

        let field_default =
            self.parse_default(self.default.as_ref(), info, ctx)?
                .map(|default_value| {
                    let field_default = default_fn_name(&ctx.name_prefix, &info.name);
                    let field_default_ident = format_ident!("{}", field_default);
                    let fn_return = maybe_optional(quote!(#enum_ident), info.required);

                    defs.push(quote! {
                        fn #field_default_ident() -> #fn_return {
                            #default_value
                        }
                    });

                    field_default
                });

        let doc_attr = doc_attribute(info.description.as_deref());

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

        Ok(FieldDef {
            field_name,
            field_rename,
            field_default,
            field_doc: info.description.clone(),
            field_ty,
            defs,
            defs_doc,
        })
    }

    fn parse_default(
        &self,
        value: Option<&Value>,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError> {
        let enum_name_without_prefix = renamed_enum(&info.name);
        let enum_name = format!("{}{}", ctx.name_prefix, enum_name_without_prefix);
        let enum_ident = format_ident!("{}", enum_name);

        value
            .map(|default| {
                default
                    .as_str()
                    .ok_or("expected default value to be an enum variant string".into())
                    .map(|variant| {
                        let (variant_name, _) = renamed_enum_variant(variant);
                        let variant_ident = format_ident!("{}", variant_name);
                        maybe_optional_value(quote!(#enum_ident::#variant_ident), info.required)
                    })
            })
            .invert()
    }
}

impl ToStruct for TupleField {
    fn to_struct(
        &self,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError> {
        let (field_name, field_rename) = renamed_field(&info.name);

        let inner_info = FieldInfo {
            required: true,
            ..info.clone()
        };

        let (mut defs, defs_doc, item_tokens) = self.items.iter().try_fold(
            (Vec::new(), Vec::new(), Vec::new()),
            |(mut defs, mut defs_doc, mut item_tokens), inner_item| {
                let FieldDef {
                    field_name: _inner_item_name,
                    field_rename: _inner_item_rename,
                    field_default: _inner_field_default,
                    field_doc: _inner_item_doc,
                    field_ty: inner_item_ty,
                    defs: inner_defs,
                    defs_doc: inner_defs_doc,
                } = inner_item.to_struct(&inner_info, ctx)?;

                defs.extend(inner_defs);
                defs_doc.extend(inner_defs_doc);

                item_tokens.push(quote!(#inner_item_ty));

                Result::<_, SchemaStructError>::Ok((defs, defs_doc, item_tokens))
            },
        )?;

        let field_ty = maybe_optional(quote!((#(#item_tokens),*)), info.required);

        let field_default =
            self.parse_default(self.default.as_ref(), info, ctx)?
                .map(|default_value| {
                    let field_default = default_fn_name(&ctx.name_prefix, &info.name);
                    let field_default_ident = format_ident!("{}", field_default);
                    let fn_return = maybe_optional(quote!((#(#item_tokens),*)), info.required);

                    defs.push(quote! {
                        fn #field_default_ident() -> #fn_return {
                            #default_value
                        }
                    });

                    field_default
                });

        Ok(FieldDef {
            field_name,
            field_rename,
            field_default,
            field_doc: info.description.clone(),
            field_ty,
            defs,
            defs_doc,
        })
    }

    fn parse_default(
        &self,
        value: Option<&Value>,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError> {
        let inner_info = FieldInfo {
            required: true,
            ..info.clone()
        };

        value
            .map(|default| {
                default
                    .as_array()
                    .ok_or("expected default value to be a tuple array".into())
                    .and_then(|values| {
                        if values.len() != self.items.len() {
                            return Err(
                                "tuple definition and default values have different lengths".into(),
                            );
                        }

                        self.items
                            .iter()
                            .enumerate()
                            .map(|(index, item)| {
                                item.parse_default(Some(&values[index]), &inner_info, ctx)
                                    .map(|inner| inner.unwrap_or(quote!(None)))
                            })
                            .collect::<Result<Vec<_>, _>>()
                            .map(|defaults| {
                                maybe_optional_value(quote!((#(#defaults),*)), info.required)
                            })
                    })
            })
            .invert()
    }
}

impl ToStruct for RefField {
    fn to_struct(
        &self,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError> {
        let (field_name, field_rename) = renamed_field(&info.name);
        let inner_schema_name = ref_name(&self.path, &ctx.root_name);
        let inner_schema_ident = format_ident!("{}", inner_schema_name);
        let field_ty = maybe_optional(quote!(Box<#inner_schema_ident>), info.required);

        Ok(FieldDef {
            field_name,
            field_rename,
            field_default: None,
            field_doc: info.description.clone(),
            field_ty,
            defs: vec![],
            defs_doc: vec![],
        })
    }

    fn parse_default(
        &self,
        _value: Option<&Value>,
        _info: &FieldInfo,
        _ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError> {
        Ok(None)
    }
}

impl ToStruct for FieldType {
    fn to_struct(
        &self,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError> {
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

    fn parse_default(
        &self,
        value: Option<&Value>,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError> {
        match self {
            Self::Null(field) => field.parse_default(value, info, ctx),
            Self::Boolean(field) => field.parse_default(value, info, ctx),
            Self::Integer(field) => field.parse_default(value, info, ctx),
            Self::Number(field) => field.parse_default(value, info, ctx),
            Self::String(field) => field.parse_default(value, info, ctx),
            Self::Array(field) => field.parse_default(value, info, ctx),
            Self::Object(field) => field.parse_default(value, info, ctx),
            Self::Enum(field) => field.parse_default(value, info, ctx),
            Self::Tuple(field) => field.parse_default(value, info, ctx),
            Self::Ref(field) => field.parse_default(value, info, ctx),
        }
    }
}

impl ToStruct for Field {
    fn to_struct(
        &self,
        _info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError> {
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

    fn parse_default(
        &self,
        value: Option<&Value>,
        _info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError> {
        let info = if self.info.subschema {
            FieldInfo {
                name: renamed_ref(&self.info.name, &ctx.root_name),
                ..self.info.clone()
            }
        } else {
            self.info.clone()
        };

        self.ty.parse_default(value, &info, ctx)
    }
}

impl ToStruct for Subschema {
    fn to_struct(
        &self,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<FieldDef, SchemaStructError> {
        let vis = &ctx.vis;
        let subschema_name = renamed_ref(&info.name, &ctx.root_name);
        let subschema_ident = format_ident!("{}", subschema_name);

        let FieldDef {
            field_name,
            field_rename,
            field_default,
            field_doc,
            field_ty,
            mut defs,
            mut defs_doc,
        } = self.schema.to_struct(info, ctx)?;

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

        Ok(FieldDef {
            field_name,
            field_rename,
            field_default,
            field_doc,
            field_ty: quote!(#subschema_ident),
            defs,
            defs_doc,
        })
    }

    fn parse_default(
        &self,
        value: Option<&Value>,
        info: &FieldInfo,
        ctx: &FieldContext,
    ) -> Result<Option<TokenStream>, SchemaStructError> {
        self.schema.parse_default(value, info, ctx)
    }
}
