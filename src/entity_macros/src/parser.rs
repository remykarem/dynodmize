use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::{Data, DeriveInput, Error, Lit, Meta};

pub fn expand_entity(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let mut pk_segments = vec![];
    let mut sk_segments = vec![];
    let mut non_keys = vec![];

    let mut pk_target_val: Option<String> = None;
    let mut sk_target_val: Option<String> = None;

    let Data::Struct(ds) = &input.data else {
        return Error::new_spanned(input, "Entity can only be derived for structs")
            .to_compile_error();
    };

    for field in ds.fields.iter() {
        let ident = field.ident.as_ref().unwrap();

        // defaults
        let mut seg_name: Option<String> = None;
        let mut seg_order: Option<usize> = None;
        let mut serialize_as_non_key = true;

        for attr in &field.attrs {
            if attr.path().is_ident("partition") || attr.path().is_ident("sort") {
                if let Meta::List(list) = attr.meta.clone() {
                    let parsed =
                        syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
                            .parse2(list.tokens.clone())
                            .unwrap();

                    for nested in parsed {
                        if let Meta::NameValue(nv) = nested {
                            let key = nv.path.get_ident().unwrap().to_string();
                            if let syn::Expr::Lit(expr_lit) = &nv.value {
                                match (&key[..], &expr_lit.lit) {
                                    ("pk_target", Lit::Str(s)) => {
                                        match &pk_target_val {
                                            None => pk_target_val = Some(s.value()),
                                            Some(existing) if existing != &s.value() => {
                                                panic!("Multiple differing pk_target values found");
                                            }
                                            _ => {}
                                        }
                                    }
                                    ("sk_target", Lit::Str(s)) => {
                                        match &sk_target_val {
                                            None => sk_target_val = Some(s.value()),
                                            Some(existing) if existing != &s.value() => {
                                                panic!("Multiple differing sk_target values found");
                                            }
                                            _ => {}
                                        }
                                    }
                                    ("pk_attribute_segment_name", Lit::Str(s))
                                    | ("sk_attribute_segment_name", Lit::Str(s)) => {
                                        seg_name = Some(s.value());
                                    }
                                    ("pk_attribute_segment_order", Lit::Int(i))
                                    | ("sk_attribute_segment_order", Lit::Int(i)) => {
                                        seg_order = Some(i.base10_parse().unwrap());
                                    }
                                    ("serialize_as_non_key", Lit::Bool(b)) => {
                                        serialize_as_non_key = b.value();
                                    }
                                    _ => {
                                        panic!("Unknown attribute: {}", key);
                                    }
                                }
                            }
                        }
                    }
                }

                if attr.path().is_ident("partition") {
                    pk_segments.push((
                        seg_order.unwrap_or(usize::MAX),
                        seg_name.clone(),
                        ident.clone(),
                        serialize_as_non_key,
                    ));
                }
                if attr.path().is_ident("sort") {
                    sk_segments.push((
                        seg_order.unwrap_or(usize::MAX),
                        seg_name.clone(),
                        ident.clone(),
                        serialize_as_non_key,
                    ));
                }
            }
        }

        // unannotated → always non-key
        if field
            .attrs
            .iter()
            .all(|a| !a.path().is_ident("partition") && !a.path().is_ident("sort"))
        {
            non_keys.push(ident.clone());
        }
    }

    // enforce pk_target
    let pk_name = pk_target_val.expect("Missing required pk_target on partition field(s)");

    // enforce sk_target uniqueness
    if sk_segments.is_empty() {
        sk_target_val = None;
    }
    // if sk_segments not empty but sk_target_val is None → panic
    if !sk_segments.is_empty() && sk_target_val.is_none() {
        panic!("Found sort key fields but no sk_target provided");
    }

    // sort pk/sk segments by order
    pk_segments.sort_by_key(|(order, _, _, _)| *order);
    sk_segments.sort_by_key(|(order, _, _, _)| *order);

    // Schema PK attributes
    let pk_attrs = pk_segments.iter().map(|(_, seg_name, ident, serialize)| {
        let seg_expr = if let Some(seg) = seg_name {
            quote! { Some(#seg.to_string()) }
        } else {
            quote! { None }
        };
        let attr_name = ident.to_string();
        quote! {
            entity_core::CompositeKeyAttribute {
                attribute_segment_name_in_key: #seg_expr,
                attribute_name_in_data: #attr_name.into(),
                serialize_value_in_data: #serialize,
            }
        }
    });

    // Schema SK attributes
    let sk_attrs = sk_segments.iter().map(|(_, seg_name, ident, serialize)| {
        let seg_expr = if let Some(seg) = seg_name {
            quote! { Some(#seg.to_string()) }
        } else {
            quote! { None }
        };
        let attr_name = ident.to_string();
        quote! {
            entity_core::CompositeKeyAttribute {
                attribute_segment_name_in_key: #seg_expr,
                attribute_name_in_data: #attr_name.into(),
                serialize_value_in_data: #serialize,
            }
        }
    });

    // Data (non-key attributes)
    let data_attrs = non_keys.iter().map(|ident| {
        let attr_name = ident.to_string();
        quote! {
            entity_core::Attribute { name: #attr_name.into() }
        }
    });

    // sk schema
    let sk_schema = if let Some(sk_name) = sk_target_val {
        quote! {
            Some(entity_core::CompositeKey {
                attribute_name: #sk_name.into(),
                attributes: vec![ #( #sk_attrs ),* ],
            })
        }
    } else {
        quote! { None }
    };

    quote! {
        impl entity_core::Entity1 for #name {
            fn get_schema() -> entity_core::Schema {
                let pk = entity_core::CompositeKey {
                    attribute_name: #pk_name.into(),
                    attributes: vec![ #( #pk_attrs ),* ],
                };

                let sk = #sk_schema;

                entity_core::Schema {
                    partition_key: pk,
                    sort_key: sk,
                    data: std::collections::HashSet::from([ #( #data_attrs ),* ]),
                    delimiter: '#',
                }
            }
        }
    }
}
