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
    quote! { Vec::<entity_core::Segment>::from([ #( #parts ),* ]) }
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
        quote! { Vec::<entity_core::KeyDef<entity_core::AttributeValue>>::from([ #( #items ),* ]) }
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
            .map(|segment| {
                let field = syn::Ident::new(&segment.struct_field_name, proc_macro2::Span::call_site());
                if let Some(pfx) = &segment.prefix {
                    quote! { format!("{}#{}", #pfx, self.#field) }
                } else {
                    quote! { self.#field.to_string() }
                }
            });
        quote! { Vec::<String>::from([ #( #segs ),* ]).join("#") }
    };

    //
    // ─── SK ──────────────────────────────────────────────
    //
    let sk_expr = if let Some(sk) = &schema.sort_key_def {
        let sk_name = &sk.attribute_name;

        match &sk.attribute_value {
            // If the sort key is static, directly insert it with the attribute name
            AttributeValue::Static(static_value) => {
                quote! {
                {
                    let mut map: ::std::collections::HashMap<String, String> = ::std::collections::HashMap::new();
                    map.insert(#sk_name.to_string(), #static_value.to_string());
                    map
                }
            }
            }
            // For composite sort keys, compute the value based on segments
            AttributeValue::Composite(CompositeAttributeValue {
                                          segments,
                                          prefix,
                                          suffix,
                                      }) => {
                // Generate the parts of the composite key based on the segments
                let segment_parts = segments.iter().map(|segment| {
                    let field = syn::Ident::new(&segment.struct_field_name, proc_macro2::Span::call_site());
                    if let Some(pfx) = &segment.prefix {
                        // Prefix is included if present
                        quote! { format!("{}#{}", #pfx, self.#field) }
                    } else {
                        // Use the field value directly if no prefix
                        quote! { self.#field.to_string() }
                    }
                });

                // Handle the optional prefix and suffix for the composite key
                let final_prefix = match prefix {
                    Some(p) => quote! { parts.push(#p.to_string()); },
                    None => quote! {},
                };
                let final_suffix = match suffix {
                    Some(s) => quote! { parts.push(#s.to_string()); },
                    None => quote! {},
                };

                quote! {
                {
                    let mut map: ::std::collections::HashMap<String, String> = ::std::collections::HashMap::new();

                    // Collect all parts of the composite sort key
                    let mut parts: Vec<String> = Vec::new();
                    #final_prefix
                    parts.extend(vec![
                        #(#segment_parts),*
                    ]);
                    #final_suffix

                    // Join the parts with "#" and insert into the map
                    let composite_sk = parts.join("#");
                    map.insert(#sk_name.to_string(), composite_sk);
                    map
                }
            }
            }
        }
    } else {
        quote! {
        {
            // Return an empty HashMap if no sort key is defined
            ::std::collections::HashMap::<String, String>::new()
        }
    }
    };    //
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

                // pk
                map.insert(#pk_attr_name.to_string(), serde_json::Value::String(#pk_expr));

                // sk
                let sk_map: ::std::collections::HashMap<String, String> = #sk_expr;
                for (key, value) in sk_map {
                    map.insert(key, serde_json::Value::String(value));
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