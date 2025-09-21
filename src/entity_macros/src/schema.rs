use crate::parser::{
    NkDef, RawFieldDef, RawNkFieldDef, RawPkFieldDef, RawPkStructDef, RawSkFieldDef,
    RawSkStructDef, RawStructFieldDefs,
};
use entity_core::{AttributeValue, CompositeAttributeValue, KeyDef, SchemaV2, Segment};
use std::collections::HashMap;
use syn::Error;

pub fn build_schema(
    pk_struct_def: Option<RawPkStructDef>,
    sk_struct_def: Option<RawSkStructDef>,
    nk_defs: Vec<NkDef>,
    field_defs: Vec<RawStructFieldDefs>,
) -> Result<SchemaV2, syn::Error> {
    // If multiple pks are defined, check if all of them have an `order`
    let pk_segments: Vec<_> = field_defs
        .iter()
        .filter_map(|field| {
            if let RawFieldDef::Pk(pk) = &field.raw_field_def {
                Some((pk.span, pk)) // Collect the span for better diagnostics
            } else {
                None
            }
        })
        .collect();

    // If there are multiple PKs, ensure each has an explicit `order` attribute
    if pk_segments.len() > 1 {
        let missing_order = pk_segments
            .iter()
            .filter(|(_, pk)| pk.order.is_none())
            .map(|(span, _)| span) // Collect the spans of problematic #[pk] attributes
            .collect::<Vec<_>>();

        if !missing_order.is_empty() {
            let mut diagnostic = syn::Error::new(
                proc_macro2::Span::call_site(),
                "Multiple primary keys defined, but some do not have an `order` attribute.",
            );

            for span in missing_order {
                diagnostic.combine(syn::Error::new(
                    *span,
                    "This `#[pk]` is missing an `order` attribute.",
                ));
            }

            return Err(diagnostic);
        }
    }

    // If multiple pks are defined, check if all of them have an `order`
    let sk_segments: Vec<_> = field_defs
        .iter()
        .filter_map(|field| {
            if let RawFieldDef::Sk(sk) = &field.raw_field_def {
                Some((sk.span, sk)) // Collect the span for better diagnostics
            } else {
                None
            }
        })
        .collect();

    // If there are multiple PKs, ensure each has an explicit `order` attribute
    if sk_segments.len() > 1 {
        let missing_order = sk_segments
            .iter()
            .filter(|(_, pk)| pk.order.is_none())
            .map(|(span, _)| span) // Collect the spans of problematic #[pk] attributes
            .collect::<Vec<_>>();

        if !missing_order.is_empty() {
            let mut diagnostic = syn::Error::new(
                proc_macro2::Span::call_site(),
                "Multiple sort keys defined, but some do not have an `order` attribute.",
            );

            for span in missing_order {
                diagnostic.combine(syn::Error::new(
                    *span,
                    "This `#[sk]` attribute is missing an `order` attribute.",
                ));
            }

            return Err(diagnostic);
        }
    }

    //
    // ─── BUILD PK ────────────────────────────────────────────────────────────────
    //
    let pk = if let Some(pk_struct_def) = pk_struct_def {
        // Source of truth for pk, other fields must conform to it
        let mut pk_segments: Vec<(Option<usize>, Segment)> = vec![];
        for field_def in &field_defs {
            if let RawFieldDef::Pk(RawPkFieldDef {
                name,
                prefix,
                order,
                span,
            }) = &field_def.raw_field_def
            {
                pk_segments.push((
                    *order,
                    Segment {
                        struct_field_name: name.clone(),
                        prefix: prefix.clone(),
                    },
                ));
            }
        }
        pk_segments.sort_by_key(|(ord, _)| *ord);
        let segments: Vec<Segment> = pk_segments.into_iter().map(|(_, seg)| seg).collect();

        KeyDef {
            attribute_name: pk_struct_def.name,
            attribute_value: CompositeAttributeValue {
                prefix: pk_struct_def.value_prefix,
                suffix: pk_struct_def.value_suffix,
                segments,
            },
        }
    } else {
        // Make sure there is one and only one PK in field infos
        let mut yo: Vec<&RawPkFieldDef> = field_defs
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
        let pk_def = yo.pop().unwrap();

        KeyDef {
            attribute_name: pk_def.name.clone(),
            attribute_value: CompositeAttributeValue {
                prefix: pk_def.prefix.clone(),
                suffix: None,
                segments: vec![Segment {
                    struct_field_name: pk_def.name.clone(),
                    prefix: pk_def.prefix.clone(),
                }],
            },
        }
    };

    //
    // ─── BUILD SK ────────────────────────────────────────────────────────────────
    //
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
    let sk = if let Some(sk_def) = sk_struct_def {
        let mut sk_segments: Vec<(Option<usize>, Segment)> = vec![];
        for field_info in &field_defs {
            if let RawFieldDef::Sk(RawSkFieldDef { prefix, order, .. }) = &field_info.raw_field_def
            {
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
        // Make sure there is at most one SK in field infos
        let mut yo: Vec<&RawSkFieldDef> = field_defs
            .iter()
            .filter_map(|field| {
                if let RawFieldDef::Sk(a) = &field.raw_field_def {
                    Some(a)
                } else {
                    None
                }
            })
            .collect();
        if yo.len() > 1 {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "If multiple fields contribute to a sk, there needs to be a struct-level #[sk(name = \"something\"]",
            ));
        }
        let sk_def = yo.pop();

        match sk_def {
            None => None,
            Some(sk_def) => Some(KeyDef {
                attribute_name: sk_def.name.clone(),
                attribute_value: AttributeValue::Composite(CompositeAttributeValue {
                    prefix: sk_def.prefix.clone(),
                    suffix: None,
                    segments: vec![Segment {
                        struct_field_name: sk_def.name.clone(),
                        prefix: sk_def.prefix.clone(),
                    }],
                }),
            }),
        }
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
            span,
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

pub fn validate_schema(schema: &SchemaV2) -> Result<(), syn::Error> {
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
