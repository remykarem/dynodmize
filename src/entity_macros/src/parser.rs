use crate::codegen::{tok_key_def, tok_optional_string, tok_segments};
use entity_core::{AttributeValue, CompositeAttributeValue, KeyDef, SchemaV2, Segment};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::collections::{HashMap, HashSet};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Error, Lit, Meta};

pub const DELIMITER: char = '#';

pub fn expand_entity(input: &DeriveInput) -> TokenStream {
    match parse_entity(input) {
        Ok(schema) => generate_impl(input, schema),
        Err(err) => err.to_compile_error(),
    }
}

fn parse_entity(input: &DeriveInput) -> Result<SchemaV2, Error> {
    let (pk_def, sk_def, nk_defs) = parse_entity_attrs(input)?;
    let field_infos = parse_struct_fields(input)?;
    let (pk_def, sk_def, field_infos) =
        validate_field_attrs_against_struct_attrs(pk_def, sk_def, field_infos)?;
    let schema = build_ir(pk_def, sk_def, nk_defs, field_infos)?;
    validate_schema(&schema)?;
    Ok(schema)
}

//
// ─── ENTITY LEVEL ATTRS ─────────────────────────────────────────────────────────
//

struct RawPkStructDef {
    name: String,
    value_prefix: Option<String>,
    value_suffix: Option<String>,
}

struct RawPkFieldDef {
    prefix: Option<String>,
    order: Option<usize>,
}

struct RawSkFieldDef {
    prefix: Option<String>,
    order: Option<usize>,
}

struct RawNkFieldDef {
    name: String,
    prefix: Option<String>,
    order: Option<usize>,
}

struct RawSkStructDef {
    name: String,
    value_prefix: Option<String>,
    value_suffix: Option<String>,
    value: Option<String>,
}

struct NkDef {
    name: String,
    value_prefix: Option<String>,
    value_suffix: Option<String>,
    static_value: Option<String>,
}

fn parse_entity_attrs(
    input: &DeriveInput,
) -> Result<(Option<RawPkStructDef>, Option<RawSkStructDef>, Vec<NkDef>), syn::Error> {
    let mut pk: Option<RawPkStructDef> = None;
    let mut sk: Option<RawSkStructDef> = None;
    let mut nks: Vec<NkDef> = vec![];

    // A struct can have multiple attributes
    for attr in &input.attrs {
        // ---------------
        // Attribute-level
        // ---------------

        // Guard
        if !(attr.path().is_ident("pk") || attr.path().is_ident("sk") || attr.path().is_ident("nk"))
        {
            continue;
        }

        if let Meta::List(list) = &attr.meta {
            // -----------------
            // #[pk(... = ...)]
            // -----------------
            let mut name: Option<String> = None;
            let mut value_prefix = None;
            let mut value_suffix = None;
            let mut static_value = None;

            let parsed =
                Punctuated::<Meta, syn::Token![,]>::parse_terminated.parse2(list.tokens.clone())?;

            for nested in parsed {
                // ---------------------
                // Field-level attribute
                // ---------------------
                if let Meta::NameValue(nv) = nested {
                    let key = nv.path.get_ident().unwrap().to_string();
                    if let syn::Expr::Lit(expr_lit) = &nv.value {
                        match (&key[..], &expr_lit.lit) {
                            ("name", Lit::Str(s)) => name = Some(s.value()),
                            ("value_prefix", Lit::Str(s)) => value_prefix = Some(s.value()),
                            ("value_suffix", Lit::Str(s)) => value_suffix = Some(s.value()),
                            ("value", Lit::Str(s)) => static_value = Some(s.value()),
                            _ => {
                                return Err(Error::new_spanned(
                                    nv,
                                    "Unknown entity-level attribute",
                                ));
                            }
                        }
                    }
                }
            }

            if attr.path().is_ident("pk") {
                if pk.is_some() {
                    return Err(Error::new_spanned(attr, "Multiple #[pk(...)] not allowed"));
                }
                let name = name
                    .clone()
                    .ok_or_else(|| Error::new_spanned(attr, "pk must have a name"))?;
                pk = Some(RawPkStructDef {
                    name,
                    value_prefix: value_prefix.clone(),
                    value_suffix: value_suffix.clone(),
                });
            }

            if attr.path().is_ident("sk") {
                if sk.is_some() {
                    return Err(Error::new_spanned(attr, "Multiple #[sk(...)] not allowed"));
                }
                let name = name
                    .clone()
                    .ok_or_else(|| Error::new_spanned(attr, "sk must have a name"))?;
                sk = Some(RawSkStructDef {
                    name,
                    value_prefix: value_prefix.clone(),
                    value_suffix: value_suffix.clone(),
                    value: static_value.clone(),
                });
            }

            if attr.path().is_ident("nk") {
                let name = name.ok_or_else(|| Error::new_spanned(attr, "nk must have a name"))?;
                nks.push(NkDef {
                    name,
                    value_prefix,
                    value_suffix,
                    static_value,
                });
            }
        }
    }

    Ok((pk, sk, nks))
}

//
// ─── FIELD LEVEL ATTRS ──────────────────────────────────────────────────────────
//

struct RawStructFieldDefs {
    field_name: String,
    raw_field_def: RawFieldDef,
}

enum RawFieldDef {
    Pk(RawPkFieldDef),
    Sk(RawSkFieldDef),
    Nk(RawNkFieldDef),
}

fn parse_struct_fields(input: &DeriveInput) -> Result<Vec<RawStructFieldDefs>, syn::Error> {
    let mut all_field_defs = vec![];

    let Data::Struct(data_struct) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "Entity can only be derived for structs",
        ));
    };

    // Every struct has several fields
    for field in &data_struct.fields {
        // -----------
        // Field-level
        // -----------

        let ident = field.ident.as_ref().unwrap();

        let mut pk_defs: Vec<RawPkFieldDef> = vec![];
        let mut sk_defs: Vec<RawSkFieldDef> = vec![];
        let mut nk_defs: Vec<RawNkFieldDef> = vec![];

        // Every field can have several attributes
        for attr in &field.attrs {
            // ---------------
            // Attribute-level
            // ---------------

            // Guard
            if !(attr.path().is_ident("pk")
                || attr.path().is_ident("sk")
                || attr.path().is_ident("nk"))
            {
                continue;
            }

            // An attribute will have one or more of these attribute fields
            let mut prefix = None;
            let mut order: Option<usize> = None;
            let mut name: Option<String> = None;

            match &attr.meta {
                // -----------------
                // #[pk(... = ...)]
                // -----------------
                Meta::List(list) => {
                    let parsed = Punctuated::<Meta, syn::Token![,]>::parse_terminated
                        .parse2(list.tokens.clone())?;

                    for nested in parsed {
                        // ---------------------
                        // Field-level attribute
                        // ---------------------
                        if let Meta::NameValue(nv) = nested {
                            let key = nv.path.get_ident().unwrap().to_string();
                            if let syn::Expr::Lit(expr_lit) = &nv.value {
                                match (&key[..], &expr_lit.lit) {
                                    ("prefix", Lit::Str(s)) => prefix = Some(s.value()),
                                    ("order", Lit::Int(i)) => order = Some(i.base10_parse()?),
                                    ("name", Lit::Str(s)) => name = Some(s.value()),
                                    _ => {
                                        return Err(Error::new_spanned(
                                            nv,
                                            "Unknown field-level attribute",
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    // let order = order.ok_or_else(|| Error::new_spanned(attr, "Missing order ="))?;

                    if attr.path().is_ident("pk") {
                        pk_defs.push(RawPkFieldDef {
                            prefix: prefix.clone(),
                            order,
                        })
                    }

                    if attr.path().is_ident("sk") {
                        sk_defs.push(RawSkFieldDef {
                            prefix: prefix.clone(),
                            order,
                        })
                    }

                    if attr.path().is_ident("nk") {
                        let name = name
                            .ok_or_else(|| Error::new_spanned(attr, "nk field must have name"))?;
                        nk_defs.push(RawNkFieldDef {
                            name: name.clone(),
                            prefix,
                            order,
                        });
                    }
                }
                // #[pk]
                Meta::Path(path) => {
                    if attr.path().is_ident("pk") {
                        pk_defs.push(RawPkFieldDef {
                            prefix: None,
                            order,
                        })
                    }

                    if attr.path().is_ident("sk") {
                        sk_defs.push(RawSkFieldDef {
                            prefix: None,
                            order,
                        })
                    }

                    if attr.path().is_ident("nk") {
                        nk_defs.push(RawNkFieldDef {
                            name: ident.to_string(),
                            prefix: None,
                            order: None,
                        });
                    }
                }
                _ => {}
            }
        }

        // Validate all the attributes for this field
        if pk_defs.len() > 1 {
            return Err(Error::new_spanned(
                field,
                "Cannot have more than 1 pk per field",
            ));
        }
        if sk_defs.len() > 1 {
            return Err(Error::new_spanned(
                field,
                "Cannot have more than 1 sk per field",
            ));
        }
        if !nk_defs.is_empty() {
            let unique_nk_names: HashSet<&str> =
                nk_defs.iter().map(|nk| nk.name.as_str()).collect();
            if unique_nk_names.len() < nk_defs.len() {
                return Err(Error::new_spanned(
                    field,
                    "Cannot assign to the same nk per field",
                ));
            }
        }

        for pk_def in pk_defs {
            all_field_defs.push(RawStructFieldDefs {
                field_name: ident.to_string(),
                raw_field_def: RawFieldDef::Pk(pk_def),
            });
        }
        for sk_def in sk_defs {
            all_field_defs.push(RawStructFieldDefs {
                field_name: ident.to_string(),
                raw_field_def: RawFieldDef::Sk(sk_def),
            });
        }
        for nk_def in nk_defs {
            all_field_defs.push(RawStructFieldDefs {
                field_name: ident.to_string(),
                raw_field_def: RawFieldDef::Nk(nk_def),
            });
        }
    }

    Ok(all_field_defs)
}

//
// ─── IR BUILD + VALIDATION ──────────────────────────────────────────────────────
//

//
// ─── CODEGEN ────────────────────────────────────────────────────────────────────
//

fn generate_impl(input: &DeriveInput, schema: SchemaV2) -> TokenStream {
    let name = &input.ident;

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
        match &sk.attribute_value {
            AttributeValue::Static(static_value) => {
                quote! {
                    #static_value.to_string()
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
                        Some(parts.join("#"))
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

                map.insert("pk".to_string(), serde_json::Value::String(#pk_expr));

                if let Some(sk_val) = #sk_expr {
                    map.insert("sk".to_string(), serde_json::Value::String(sk_val));
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

fn build_ir(
    pk_struct_def: RawPkStructDef,
    sk_struct_def: Option<RawSkStructDef>,
    nk_defs: Vec<NkDef>,
    field_defs: Vec<RawStructFieldDefs>,
) -> Result<SchemaV2, syn::Error> {
    //
    // ─── BUILD PK ────────────────────────────────────────────────────────────────
    //
    let mut pk_segments: Vec<(Option<usize>, Segment)> = vec![];
    for f in &field_defs {
        if let RawFieldDef::Pk(RawPkFieldDef { prefix, order }) = &f.raw_field_def {
            pk_segments.push((
                *order,
                Segment {
                    struct_field_name: f.field_name.clone(),
                    prefix: prefix.clone(),
                },
            ));
        }
    }
    pk_segments.sort_by_key(|(ord, _)| *ord);
    let pk_segments: Vec<Segment> = pk_segments.into_iter().map(|(_, seg)| seg).collect();

    let pk = KeyDef {
        attribute_name: pk_struct_def.name,
        attribute_value: CompositeAttributeValue {
            prefix: pk_struct_def.value_prefix,
            suffix: pk_struct_def.value_suffix,
            segments: pk_segments,
        },
    };

    //
    // ─── BUILD SK ────────────────────────────────────────────────────────────────
    //
    let sk = if let Some(sk_def) = sk_struct_def {
        let mut sk_segments: Vec<(Option<usize>, Segment)> = vec![];
        for field_info in &field_defs {
            if let RawFieldDef::Sk(RawSkFieldDef { prefix, order }) = &field_info.raw_field_def {
                sk_segments.push((
                    *order,
                    Segment {
                        struct_field_name: field_info.field_name.clone(),
                        prefix: prefix.clone(),
                    },
                ));
            }
        }
        sk_segments.sort_by_key(|(ord, _)| *ord);
        let sk_segments: Vec<Segment> = sk_segments.into_iter().map(|(_, seg)| seg).collect();

        Some(KeyDef {
            attribute_name: sk_def.name,
            attribute_value: AttributeValue::Composite(CompositeAttributeValue {
                prefix: sk_def.value_prefix,
                suffix: sk_def.value_suffix,
                segments: sk_segments,
            }),
        })
    } else {
        None
    };

    //
    // ─── BUILD NKS ───────────────────────────────────────────────────────────────
    //
    let mut nk_map: HashMap<String, KeyDef<AttributeValue>> = HashMap::new();

    // start with struct-level NKs
    for nk_def in nk_defs {
        if let Some(v) = nk_def.static_value {
            nk_map.insert(
                nk_def.name.clone(),
                KeyDef {
                    attribute_name: nk_def.name,
                    attribute_value: AttributeValue::Static(v),
                },
            );
        } else {
            nk_map.insert(
                nk_def.name.clone(),
                KeyDef {
                    attribute_name: nk_def.name,
                    attribute_value: AttributeValue::Composite(CompositeAttributeValue {
                        prefix: nk_def.value_prefix,
                        suffix: nk_def.value_suffix,
                        segments: vec![],
                    }),
                },
            );
        }
    }

    // add field-level NKs
    for field_info in &field_defs {
        if let RawFieldDef::Nk(RawNkFieldDef {
            name,
            prefix,
            order,
        }) = &field_info.raw_field_def
        {
            let nn = name.clone();
            let entry = nk_map.entry(nn).or_insert(KeyDef {
                attribute_name: name.clone(),
                attribute_value: AttributeValue::Composite(CompositeAttributeValue {
                    prefix: None,
                    suffix: None,
                    segments: vec![],
                }),
            });

            match &mut entry.attribute_value {
                AttributeValue::Static(_) => {
                    return Err(Error::new_spanned(
                        &field_info.field_name,
                        format!(
                            "NK {} is defined as static at struct level and cannot have field segments",
                            name
                        ),
                    ));
                }
                AttributeValue::Composite(CompositeAttributeValue { segments, .. }) => {
                    segments.push(Segment {
                        struct_field_name: field_info.field_name.clone(),
                        prefix: prefix.clone(),
                    });
                }
            }
        }
    }

    // sort NK segments by order and flatten
    let mut non_keys: Vec<KeyDef<AttributeValue>> = vec![];
    for (_, mut nk) in nk_map {
        if let AttributeValue::Composite(CompositeAttributeValue { segments, .. }) =
            &mut nk.attribute_value
        {
            let mut ordered: Vec<Segment> = vec![];
            for seg in std::mem::take(segments) {
                ordered.push(seg);
            }
        }
        non_keys.push(nk);
    }

    Ok(SchemaV2 {
        partition_key_def: pk,
        sort_key_def: sk,
        non_key_defs: non_keys,
    })
}

fn validate_schema(schema: &SchemaV2) -> Result<(), syn::Error> {
    // validate pk
    if schema.partition_key_def.attribute_value.segments.is_empty() {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "PK must have segments or static value",
        ));
    }

    // validate sk
    // if let Some(sk) = &schema.sort_key_def {
    //     if let AttributeValue::Composite(CompositeAttributeValue { segments, .. }) =
    //         &sk.attribute_value
    //     {
    //         if segments.is_empty() {
    //             return Err(Error::new(
    //                 proc_macro2::Span::call_site(),
    //                 "SK must have segments or static valueee",
    //             ));
    //         }
    //     }
    // }

    // validate NKs
    for nk in &schema.non_key_defs {
        if let AttributeValue::Composite(CompositeAttributeValue { segments, .. }) =
            &nk.attribute_value
        {
            if segments.is_empty() {
                return Err(Error::new(
                    proc_macro2::Span::call_site(),
                    "NK must have segments or static valueee",
                ));
            }
        }
    }

    Ok(())
}

fn validate_field_attrs_against_struct_attrs(
    pk_struct_def: Option<RawPkStructDef>,
    sk_struct_def: Option<RawSkStructDef>,
    field_defs: Vec<RawStructFieldDefs>,
) -> Result<
    (
        RawPkStructDef,
        Option<RawSkStructDef>,
        Vec<RawStructFieldDefs>,
    ),
    syn::Error,
> {
    let pk_def = if let Some(pk_def) = pk_struct_def {
        pk_def
    } else {
        // Make sure there is one and only one PK in field infos
        let yo: Vec<&RawPkFieldDef> = field_defs
            .iter()
            .filter_map(|field| {
                if let RawFieldDef::Pk(a) = &field.raw_field_def {
                    Some(a)
                } else {
                    None
                }
            })
            .collect();
        if yo.len() != 1 {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "If multiple fields contribute to a pk, there needs to be a struct-level #[pk(name = \"something\"]",
            ));
        }

        RawPkStructDef {
            name: "".to_string(),
            value_prefix: None,
            value_suffix: None,
        }
    };

    // let sk_fields: Vec<&RawStructFieldDefs> =
    //     field_defs.iter().filter(|f| f.raw_field_def.is_some()).collect();
    // if !sk_fields.is_empty() && sk_struct_def.is_none() {
    //     return Err(syn::Error::new(
    //         proc_macro2::Span::call_site(),
    //         "Fields annotated with #[sk] require a struct-level #[sk(name=...)]",
    //     ));
    // }

    // Check if order is specified for sk fields
    // if sk_fields.len() > 1 {
    //     for sk_field in sk_fields {
    //         if let Some(RawSkFieldDef { .. }) = sk_field.sk_def.as_ref() {
    //             return Err(syn::Error::new(
    //                 proc_macro2::Span::call_site(),
    //                 "If more than one field contribute to #[sk], #[sk(order=...)]",
    //             ));
    //         }
    //     }
    // }

    Ok((pk_def, sk_struct_def, field_defs))
}
