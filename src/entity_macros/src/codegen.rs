use entity_core::{AttributeValue, CompositeAttributeValue, SchemaV2, Segment};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn tok_optional_string(v: &Option<String>) -> TokenStream {
    match v {
        Some(s) => quote! { Some(#s.to_string()) },
        None => quote! { None },
    }
}

pub fn tok_key_def(attribute_name: &str, attribute_value: &AttributeValue) -> TokenStream {
    match attribute_value {
        AttributeValue::Static(static_name) => {
            quote! {
                entity_core::KeyDef {
                    attribute_name: #attribute_name.to_string(),
                    attribute_value: entity_core::AttributeValue::Static(#static_name.to_string()),
                }
            }
        }
        AttributeValue::Composite(composite_attribute_value) => {
            let sk_vp = tok_optional_string(&composite_attribute_value.prefix);
            let sk_vs = tok_optional_string(&composite_attribute_value.suffix);
            let sk_segments = tok_segments(&composite_attribute_value.segments);
            quote! {
                entity_core::KeyDef {
                    attribute_name: #attribute_name.to_string(),
                    attribute_value: entity_core::AttributeValue::Composite(entity_core::CompositeAttributeValue {
                        segments: #sk_segments,
                        prefix: #sk_vp,
                        suffix: #sk_vs,
                    }),
                }
            }
        }
    }
}

pub(crate) fn tok_segments(segments: &[Segment]) -> TokenStream {
    let parts = segments.iter().map(|segment| {
        let field = &segment.struct_field_name;
        match &segment.prefix {
            Some(p) => quote! {
                entity_core::Segment {
                    struct_field_name: #field.to_string(),
                    prefix: Some(#p.to_string()),
                }
            },
            None => quote! {
                entity_core::Segment {
                    struct_field_name: #field.to_string(),
                    prefix: None,
                }
            },
        }
    });
    quote! { vec![ #( #parts ),* ] }
}

pub fn generate_impl(input: &DeriveInput, schema: SchemaV2) -> TokenStream {
    // --- PK tokens ---
    let pk_attr_name = schema.partition_key_def.attribute_name;
    let pk_vp = tok_optional_string(&schema.partition_key_def.attribute_value.prefix);
    let pk_vs = tok_optional_string(&schema.partition_key_def.attribute_value.suffix);
    let pk_segments = tok_segments(&schema.partition_key_def.attribute_value.segments);
    let partition_key_def_tokens = quote! {
            entity_core::KeyDef {
                attribute_name: #pk_attr_name.to_string(),
                attribute_value: entity_core::CompositeAttributeValue {
                    segments: #pk_segments,
                    prefix: #pk_vp,
                    suffix: #pk_vs,
                },
            }
    };

    // --- SK tokens (optional) ---
    let sort_key_def_tokens = if let Some(sk_def) = &schema.sort_key_def {
        let sk_name = sk_def.attribute_name.clone();
        let key_def = tok_key_def(&sk_name, &sk_def.attribute_value);
        quote! { Some(#key_def) }
    } else {
        quote! { None }
    };

    // --- NK tokens ---
    let nk_items = {
        let items = schema.non_key_defs.iter().map(|nk| {
            let name = &nk.attribute_name;
            tok_key_def(name, &nk.attribute_value)
        });
        quote! { vec![ #( #items ),* ] }
    };

    let name = &input.ident;

    //
    // ─── PK ──────────────────────────────────────────────
    //
    let pk_expr = {
        let segs = schema
            .partition_key_def
            .attribute_value
            .segments
            .iter()
            .map(|seg| {
                let field = syn::Ident::new(&seg.struct_field_name, proc_macro2::Span::call_site());
                if let Some(pfx) = &seg.prefix {
                    quote! { format!("{}#{}", #pfx, self.#field) }
                } else {
                    quote! { self.#field.to_string() }
                }
            });
        quote! { vec![ #( #segs ),* ].join("#") }
    };

    //
    // ─── SK ──────────────────────────────────────────────
    //
    let sk_expr = if let Some(sk) = &schema.sort_key_def {
        let sk_name = &sk.attribute_name;
        match &sk.attribute_value {
            AttributeValue::Static(static_value) => {
                quote! {
                    Some((#sk_name.to_string(), #static_value.to_string()))
                }
            }
            AttributeValue::Composite(CompositeAttributeValue {
                segments,
                prefix,
                suffix,
            }) => {
                let segs = segments.iter().map(|seg| {
                    let field =
                        syn::Ident::new(&seg.struct_field_name, proc_macro2::Span::call_site());

                    let vs_expr = schema
                        .partition_key_def
                        .attribute_value
                        .suffix // TODO incorrect
                        .as_ref()
                        .map(|s| quote! { parts.push(#s.to_string()); });
                    if let Some(pfx) = &seg.prefix {
                        quote! {
                            format!("{}#{}", #pfx, self.#field)
                        }
                    } else {
                        quote! { self.#field.to_string() }
                    }
                });
                quote! {
                    {
                        let parts: ::std::vec::Vec<::std::string::String> = vec![ #( #segs ),* ];
                        Some((#sk_name.to_string(), parts.join("#")))
                    }
                }
            }
        }
    } else {
        quote! { None }
    };

    //
    // ─── NKS ─────────────────────────────────────────────
    //
    let nk_inserts = schema.non_key_defs.iter().map(|nk| {
        let name = &nk.attribute_name;
        match &nk.attribute_value {
            AttributeValue::Static(v) => {
                let val = v.clone();
                quote! {
                    map.insert(#name.to_string(), serde_json::Value::String(#val.to_string()));
                }
            }
            AttributeValue::Composite(CompositeAttributeValue { segments, .. }) => {
                let segs = segments.iter().map(|seg| {
                    let field =
                        syn::Ident::new(&seg.struct_field_name, proc_macro2::Span::call_site());
                    if let Some(pfx) = &seg.prefix {
                        quote! { format!("{}#{}", #pfx, self.#field) }
                    } else {
                        quote! { self.#field.to_string() }
                    }
                });
                quote! {
                    map.insert(#name.to_string(),
                        serde_json::Value::String(vec![ #( #segs ),* ].join("#")));
                }
            }
        }
    });

    // --- final impl ---
    quote! {
        impl entity_core::Entity2 for #name {
            fn get_schema() -> entity_core::SchemaV2 {
                let partition_key_def = #partition_key_def_tokens;
                let sort_key_def = #sort_key_def_tokens;
                let non_key_defs: Vec<entity_core::KeyDef<entity_core::AttributeValue>> = #nk_items;

                entity_core::SchemaV2 {
                    partition_key_def,
                    sort_key_def,
                    non_key_defs,
                }
            }

            fn to_item(&self) -> serde_json::Value {
                let mut map = serde_json::Map::new();

                map.insert(#pk_attr_name.to_string(), serde_json::Value::String(#pk_expr));

                let sk_expr = #sk_expr;
                if let Some((attr_name, sk_val)) = sk_expr {
                    map.insert(attr_name, serde_json::Value::String(sk_val));
                }

                // NKs
                #( #nk_inserts )*

                serde_json::Value::Object(map)
            }
        }
    }
}

// fn to_item(&self) -> serde_json::Value {
//     let mut map = serde_json::Map::new();
//
//     // PK
//     let pk_val = #pk_expr;
//     map.insert("pk".to_string(), serde_json::Value::String(pk_val));
//
//     // SK
//     if let Some(sk_val) = #sk_expr {
//         map.insert("sk".to_string(), serde_json::Value::String(sk_val));
//     }
//
//     // NKs
//     #( #nk_inserts )*
//
//         serde_json::Value::Object(map)
// }