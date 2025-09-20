use std::collections::HashMap;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::{Data, DeriveInput, Error, Lit, Meta};
use entity_core::{CompositeKey, KeySegment, NonKey, NonKeyKind, NonKeySegment, Schema};

pub const DELIMITER: char = '#';

pub fn expand_entity(input: &DeriveInput) -> TokenStream {
    match parse_entity(input) {
        Ok(schema) => generate_impl(input, schema),
        Err(err) => err.to_compile_error(),
    }
}

fn parse_entity(input: &DeriveInput) -> Result<Schema, Error> {
    let (pk_def, sk_def, nk_defs) = parse_entity_attrs(input)?;
    let field_infos = parse_fields(input)?;
    let schema = build_ir(pk_def, sk_def, nk_defs, field_infos)?;
    validate_schema(&schema)?;
    Ok(schema)
}

//
// ─── ENTITY LEVEL ATTRS ─────────────────────────────────────────────────────────
//

struct PkDef {
    name: String,
    value_prefix: Option<String>,
    value_suffix: Option<String>,
    static_value: Option<String>,
}

struct SkDef {
    name: String,
    value_prefix: Option<String>,
    value_suffix: Option<String>,
    static_value: Option<String>,
}

struct NkDef {
    name: String,
    value_prefix: Option<String>,
    value_suffix: Option<String>,
    static_value: Option<String>,
}

fn parse_entity_attrs(
    input: &DeriveInput,
) -> Result<(PkDef, Option<SkDef>, Vec<NkDef>), syn::Error> {
    let mut pk: Option<PkDef> = None;
    let mut sk: Option<SkDef> = None;
    let mut nks: Vec<NkDef> = vec![];

    for attr in &input.attrs {
        if attr.path().is_ident("pk") || attr.path().is_ident("sk") || attr.path().is_ident("nk") {
            let meta = attr.meta.clone();

            if let Meta::List(list) = meta {
                let mut name: Option<String> = None;
                let mut value_prefix = None;
                let mut value_suffix = None;
                let mut static_value = None;

                let parsed = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
                    .parse2(list.tokens.clone())?;

                for nested in parsed {
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
                    pk = Some(PkDef {
                        name,
                        value_prefix: value_prefix.clone(),
                        value_suffix: value_suffix.clone(),
                        static_value: static_value.clone(),
                    });
                }

                if attr.path().is_ident("sk") {
                    if sk.is_some() {
                        return Err(Error::new_spanned(&attr, "Multiple #[sk(...)] not allowed"));
                    }
                    let name = name
                        .clone()
                        .ok_or_else(|| Error::new_spanned(attr, "sk must have a name"))?;
                    sk = Some(SkDef {
                        name,
                        value_prefix: value_prefix.clone(),
                        value_suffix: value_suffix.clone(),
                        static_value: static_value.clone(),
                    });
                }

                if attr.path().is_ident("nk") {
                    let name =
                        name.ok_or_else(|| Error::new_spanned(attr, "nk must have a name"))?;
                    nks.push(NkDef {
                        name,
                        value_prefix,
                        value_suffix,
                        static_value,
                    });
                }
            }
        }
    }

    let pk = pk.ok_or_else(|| Error::new_spanned(input, "Missing #[pk(...)] at entity level"))?;
    Ok((pk, sk, nks))
}

//
// ─── FIELD LEVEL ATTRS ──────────────────────────────────────────────────────────
//

struct FieldInfo {
    field_name: String,
    pk: Option<(Option<String>, usize)>, // (prefix, order)
    sk: Option<(Option<String>, usize)>,
    nks: Vec<(String, Option<String>, usize)>, // (nk name, prefix, order)
}

fn parse_fields(input: &DeriveInput) -> Result<Vec<FieldInfo>, syn::Error> {
    let mut out = vec![];

    let Data::Struct(ds) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "Entity can only be derived for structs",
        ));
    };

    for field in &ds.fields {
        let ident = field.ident.as_ref().unwrap();
        let mut pk = None;
        let mut sk = None;
        let mut nks = vec![];

        for attr in &field.attrs {
            if attr.path().is_ident("pk")
                || attr.path().is_ident("sk")
                || attr.path().is_ident("nk")
            {
                let meta = attr.meta.clone();
                if let Meta::List(list) = meta {
                    let mut prefix = None;
                    let mut order: Option<usize> = None;
                    let mut name: Option<String> = None;

                    let parsed =
                        syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
                            .parse2(list.tokens.clone())?;

                    for nested in parsed {
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

                    let order = order.ok_or_else(|| Error::new_spanned(attr, "Missing order"))?;

                    if attr.path().is_ident("pk") {
                        pk = Some((prefix.clone(), order));
                    }

                    if attr.path().is_ident("sk") {
                        sk = Some((prefix.clone(), order));
                    }

                    if attr.path().is_ident("nk") {
                        let name = name
                            .ok_or_else(|| Error::new_spanned(attr, "nk field must have name"))?;
                        nks.push((name, prefix, order));
                    }
                }
            }
        }

        out.push(FieldInfo {
            field_name: ident.to_string(),
            pk,
            sk,
            nks,
        });
    }

    Ok(out)
}

//
// ─── IR BUILD + VALIDATION ──────────────────────────────────────────────────────
//

//
// ─── CODEGEN ────────────────────────────────────────────────────────────────────
//

fn generate_impl(input: &DeriveInput, schema: Schema) -> TokenStream {

    let name = &input.ident;

    // --- small helpers to turn IR pieces into tokens ---
    let opt_str = |v: &Option<String>| -> TokenStream {
        match v {
            Some(s) => quote! { Some(#s.to_string()) },
            None => quote! { None },
        }
    };

    let seg_tokens = |segs: &Vec<KeySegment>| -> TokenStream {
        let parts = segs.iter().map(|s| {
            let field = &s.field_name;
            match &s.prefix {
                Some(p) => quote! {
                    entity_core::KeySegment {
                        field_name: #field.to_string(),
                        prefix: Some(#p.to_string()),
                    }
                },
                None => quote! {
                    entity_core::KeySegment {
                        field_name: #field.to_string(),
                        prefix: None,
                    }
                },
            }
        });
        quote! { vec![ #( #parts ),* ] }
    };

    let nk_kind_tokens = |k: &NonKeyKind| -> TokenStream {
        match k {
            NonKeyKind::Static(v) => {
                quote! { entity_core::NonKeyKind::Static(#v.to_string()) }
            }
            NonKeyKind::Composite { value_prefix, value_suffix, segments } => {
                let vp = opt_str(value_prefix);
                let vs = opt_str(value_suffix);

                // NonKey segments are NonKeySegment (not KeySegment)
                let segs = {
                    let parts = segments.iter().map(|s: &NonKeySegment| {
                        let field = &s.field_name;
                        match &s.prefix {
                            Some(p) => quote! {
                                entity_core::NonKeySegment {
                                    field_name: #field.to_string(),
                                    prefix: Some(#p.to_string()),
                                }
                            },
                            None => quote! {
                                entity_core::NonKeySegment {
                                    field_name: #field.to_string(),
                                    prefix: None,
                                }
                            },
                        }
                    });
                    quote! { vec![ #( #parts ),* ] }
                };

                quote! {
                    entity_core::NonKeyKind::Composite {
                        value_prefix: #vp,
                        value_suffix: #vs,
                        segments: #segs,
                    }
                }
            }
        }
    };

    // --- PK tokens ---
    let pk_attr_name = schema.partition_key.attribute_name;
    let pk_vp = opt_str(&schema.partition_key.value_prefix);
    let pk_vs = opt_str(&schema.partition_key.value_suffix);
    let pk_static = opt_str(&schema.partition_key.static_value);
    let pk_segments = seg_tokens(&schema.partition_key.segments);

    // --- SK tokens (optional) ---
    let sk_tokens = if let Some(sk) = &schema.sort_key {
        let sk_name = sk.attribute_name.clone();
        let sk_vp = opt_str(&sk.value_prefix);
        let sk_vs = opt_str(&sk.value_suffix);
        let sk_static = opt_str(&sk.static_value);
        let sk_segments = seg_tokens(&sk.segments);
        quote! {
            Some(entity_core::CompositeKey {
                attribute_name: #sk_name.to_string(),
                value_prefix: #sk_vp,
                value_suffix: #sk_vs,
                static_value: #sk_static,
                segments: #sk_segments,
            })
        }
    } else {
        quote! { None }
    };

    // --- NK tokens ---
    let nk_items = {
        let items = schema.non_keys.iter().map(|nk: &NonKey| {
            let name = nk.attribute_name.clone();
            let kind = nk_kind_tokens(&nk.kind);
            quote! {
                entity_core::NonKey {
                    attribute_name: #name.to_string(),
                    kind: #kind,
                }
            }
        });
        quote! { vec![ #( #items ),* ] }
    };

    let name = &input.ident;

    //
    // ─── PK ──────────────────────────────────────────────
    //
    let pk_expr = if let Some(static_value) = &schema.partition_key.static_value {
        let vp = schema.partition_key.value_prefix.clone();
        let vs = schema.partition_key.value_suffix.clone();
        let val = static_value.clone();
        let vp_expr = match &schema.partition_key.value_prefix {
            Some(p) => Some(quote! { parts.push(#p.to_string()); }),
            None => None,
        };
        let vs_expr = match &schema.partition_key.value_suffix {
            Some(s) => Some(quote! { parts.push(#s.to_string()); }),
            None => None,
        };
        quote! {
            {
                let mut parts = vec![];
                #vp_expr
                parts.push(#val.to_string());
                #vs_expr
                parts.join("#")
            }
        }
    } else {
        let segs = schema.partition_key.segments.iter().map(|seg| {
            let field = syn::Ident::new(&seg.field_name, proc_macro2::Span::call_site());
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
    let sk_expr = if let Some(sk) = &schema.sort_key {
        if let Some(static_value) = &sk.static_value {
            let vp = sk.value_prefix.clone();
            let vs = sk.value_suffix.clone();
            let val = static_value.clone();
            let vp_expr = match &schema.partition_key.value_prefix {
                Some(p) => Some(quote! { parts.push(#p.to_string()); }),
                None => None,
            };
            let vs_expr = match &schema.partition_key.value_suffix {
                Some(s) => Some(quote! { parts.push(#s.to_string()); }),
                None => None,
            };
            quote! {
                {
                    let mut parts = vec![];
                    #vp_expr
                    parts.push(#val.to_string());
                    #vs_expr
                    parts.join("#")
                }
            }
        } else {
            let segs = sk.segments.iter().map(|seg| {
                let field = syn::Ident::new(&seg.field_name, proc_macro2::Span::call_site());
                if let Some(pfx) = &seg.prefix {
                    quote! { format!("{}#{}", #pfx, self.#field) }
                } else {
                    quote! { self.#field.to_string() }
                }
            });
            quote! { Some(vec![ #( #segs ),* ].join("#")) }
        }
    } else {
        quote! { None }
    };

    //
    // ─── NKS ─────────────────────────────────────────────
    //
    let nk_inserts = schema.non_keys.iter().map(|nk| {
        let name = &nk.attribute_name;
        match &nk.kind {
            NonKeyKind::Static(v) => {
                let val = v.clone();
                quote! {
                    map.insert(#name.to_string(), serde_json::Value::String(#val.to_string()));
                }
            }
            NonKeyKind::Composite { segments, .. } => {
                let segs = segments.iter().map(|seg| {
                    let field = syn::Ident::new(&seg.field_name, proc_macro2::Span::call_site());
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
            fn get_schema() -> entity_core::Schema {
                let pk = entity_core::CompositeKey {
                    attribute_name: #pk_attr_name.to_string(),
                    value_prefix: #pk_vp,
                    value_suffix: #pk_vs,
                    static_value: #pk_static,
                    segments: #pk_segments,
                };

                let sk = #sk_tokens;

                entity_core::Schema {
                    partition_key: pk,
                    sort_key: sk,
                    non_keys: #nk_items,
                }
            }

            fn to_item(&self) -> serde_json::Value {
                let mut map = serde_json::Map::new();

                // PK
                let pk_val = #pk_expr;
                map.insert("pk".to_string(), serde_json::Value::String(pk_val));

                // SK
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

fn build_ir(
    pk_def: PkDef,
    sk_def: Option<SkDef>,
    nk_defs: Vec<NkDef>,
    field_infos: Vec<FieldInfo>,
) -> Result<Schema, syn::Error> {
    //
    // ─── BUILD PK ────────────────────────────────────────────────────────────────
    //
    let mut pk_segments: Vec<(usize, KeySegment)> = vec![];
    for f in &field_infos {
        if let Some((prefix, order)) = &f.pk {
            pk_segments.push((
                *order,
                KeySegment {
                    field_name: f.field_name.clone(),
                    prefix: prefix.clone(),
                },
            ));
        }
    }
    pk_segments.sort_by_key(|(ord, _)| *ord);
    let pk_segments: Vec<KeySegment> = pk_segments.into_iter().map(|(_, seg)| seg).collect();

    let pk = CompositeKey {
        attribute_name: pk_def.name,
        value_prefix: pk_def.value_prefix,
        value_suffix: pk_def.value_suffix,
        static_value: pk_def.static_value,
        segments: pk_segments,
    };

    //
    // ─── BUILD SK ────────────────────────────────────────────────────────────────
    //
    let sk = if let Some(sk_def) = sk_def {
        let mut sk_segments: Vec<(usize, KeySegment)> = vec![];
        for f in &field_infos {
            if let Some((prefix, order)) = &f.sk {
                sk_segments.push((
                    *order,
                    KeySegment {
                        field_name: f.field_name.clone(),
                        prefix: prefix.clone(),
                    },
                ));
            }
        }
        sk_segments.sort_by_key(|(ord, _)| *ord);
        let sk_segments: Vec<KeySegment> = sk_segments.into_iter().map(|(_, seg)| seg).collect();

        Some(CompositeKey {
            attribute_name: sk_def.name,
            value_prefix: sk_def.value_prefix,
            value_suffix: sk_def.value_suffix,
            static_value: sk_def.static_value,
            segments: sk_segments,
        })
    } else {
        None
    };

    //
    // ─── BUILD NKS ───────────────────────────────────────────────────────────────
    //
    let mut nk_map: HashMap<String, NonKey> = HashMap::new();

    // start with struct-level NKs
    for nk_def in nk_defs {
        if let Some(v) = nk_def.static_value {
            nk_map.insert(
                nk_def.name.clone(),
                NonKey {
                    attribute_name: nk_def.name,
                    kind: NonKeyKind::Static(v),
                },
            );
        } else {
            nk_map.insert(
                nk_def.name.clone(),
                NonKey {
                    attribute_name: nk_def.name,
                    kind: NonKeyKind::Composite {
                        value_prefix: nk_def.value_prefix,
                        value_suffix: nk_def.value_suffix,
                        segments: vec![],
                    },
                },
            );
        }
    }

    // add field-level NKs
    for field_info in &field_infos {
        for (nk_name, prefix, _order) in &field_info.nks {
            let entry = nk_map.entry(nk_name.clone()).or_insert(NonKey {
                attribute_name: nk_name.clone(),
                kind: NonKeyKind::Composite {
                    value_prefix: None,
                    value_suffix: None,
                    segments: vec![],
                },
            });

            match &mut entry.kind {
                NonKeyKind::Static(_) => {
                    return Err(Error::new_spanned(
                        &field_info.field_name,
                        format!(
                            "NK {} is defined as static at struct level and cannot have field segments",
                            nk_name
                        ),
                    ));
                }
                NonKeyKind::Composite { segments, .. } => {
                    segments.push(NonKeySegment {
                        field_name: field_info.field_name.clone(),
                        prefix: prefix.clone(),
                    });
                }
            }
        }
    }

    // sort NK segments by order and flatten
    let mut non_keys: Vec<NonKey> = vec![];
    for (_, mut nk) in nk_map {
        if let NonKeyKind::Composite { segments, .. } = &mut nk.kind {
            let mut ordered: Vec<NonKeySegment> = vec![];
            for seg in std::mem::take(segments) {
                ordered.push(seg);
            }
        }
        non_keys.push(nk);
    }

    Ok(Schema {
        partition_key: pk,
        sort_key: sk,
        non_keys,
    })
}

fn validate_schema(schema: &Schema) -> Result<(), syn::Error> {
    // validate pk
    if schema.partition_key.static_value.is_none() && schema.partition_key.segments.is_empty() {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "PK must have segments or static value",
        ));
    }

    // validate sk
    if let Some(sk) = &schema.sort_key {
        if sk.static_value.is_none() && sk.segments.is_empty() {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "SK must have segments or static value",
            ));
        }
    }

    // validate NKs
    for nk in &schema.non_keys {
        if let NonKeyKind::Composite { segments, .. } = &nk.kind {
            if segments.is_empty() {
                return Err(Error::new(
                    proc_macro2::Span::call_site(),
                    format!("NK {} has no segments", nk.attribute_name),
                ));
            }
        }
    }

    Ok(())
}
