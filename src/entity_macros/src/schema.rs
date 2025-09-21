use crate::parser::{
    RawFieldDef, RawNkFieldDef, RawNkStructDef, RawPkFieldDef, RawPkStructDef, RawSkFieldDef,
    RawSkStructDef, RawStructFieldDefs,
};
use entity_core::{AttributeValue, CompositeAttributeValue, KeyDef, SchemaV2, Segment};
use std::collections::HashMap;
use syn::Error;

pub fn build_schema(
    pk_struct_def: Option<RawPkStructDef>,
    sk_struct_def: Option<RawSkStructDef>,
    nk_struct_defs: Vec<RawNkStructDef>,
    all_field_defs: Vec<RawStructFieldDefs>,
) -> Result<SchemaV2, syn::Error> {
    let pk_field_defs: Vec<&RawPkFieldDef> = all_field_defs
        .iter()
        .filter_map(|field| {
            if let RawStructFieldDefs::Pk(pk) = &field {
                Some(pk)
            } else {
                None
            }
        })
        .collect();
    let sk_field_defs: Vec<&RawSkFieldDef> = all_field_defs
        .iter()
        .filter_map(|field| {
            if let RawStructFieldDefs::Sk(sk) = &field {
                Some(sk)
            } else {
                None
            }
        })
        .collect();
    let nk_field_defs: Vec<&RawNkFieldDef> = all_field_defs
        .iter()
        .filter_map(|field| {
            if let RawStructFieldDefs::Nk(nk) = &field {
                Some(nk)
            } else {
                None
            }
        })
        .collect();

    // If there are multiple PKs, ensure each has an explicit `order` attribute
    if pk_field_defs.len() > 1 {
        let missing_order = pk_field_defs
            .iter()
            .filter(|pk| pk.order.is_none())
            .map(|pk| pk.span) // Collect the spans of problematic #[pk] attributes
            .collect::<Vec<_>>();

        if !missing_order.is_empty() {
            let mut diagnostic = syn::Error::new(
                proc_macro2::Span::call_site(),
                "Multiple primary keys defined, but some do not have an `order` attribute.",
            );

            for span in missing_order {
                diagnostic.combine(syn::Error::new(
                    span,
                    "This `#[pk]` is missing an `order` attribute.",
                ));
            }

            return Err(diagnostic);
        }
    }

    // If there are multiple SKs, ensure each has an explicit `order` attribute
    if sk_field_defs.len() > 1 {
        let missing_order = sk_field_defs
            .iter()
            .filter(|sk| sk.order.is_none())
            .map(|sk| sk.span) // Collect the spans of problematic #[pk] attributes
            .collect::<Vec<_>>();

        if !missing_order.is_empty() {
            let mut diagnostic = syn::Error::new(
                proc_macro2::Span::call_site(),
                "Multiple sort keys defined, but some do not have an `order` attribute.",
            );

            for span in missing_order {
                diagnostic.combine(syn::Error::new(
                    span,
                    "This `#[sk]` attribute is missing an `order` attribute.",
                ));
            }

            return Err(diagnostic);
        }
    }

    //
    // ─── BUILD PK ────────────────────────────────────────────────────────────────
    //
    let partition_key_def = if let Some(pk_struct_def) = pk_struct_def {
        // Source of truth for pk, other fields must conform to it
        let mut pk_segments: Vec<(Option<usize>, Segment)> = vec![];
        for pk_field_def in pk_field_defs {
            let RawPkFieldDef {
                field_name: name,
                prefix,
                order,
                ..
            } = &pk_field_def;
            pk_segments.push((
                *order,
                Segment {
                    struct_field_name: name.clone(),
                    prefix: prefix.clone(),
                },
            ));
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
        let mut yo: Vec<&RawPkFieldDef> = all_field_defs
            .iter()
            .filter_map(|field| {
                if let RawStructFieldDefs::Pk(a) = &field {
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
            attribute_name: pk_def.field_name.clone(),
            attribute_value: CompositeAttributeValue {
                prefix: pk_def.prefix.clone(),
                suffix: None,
                segments: vec![Segment {
                    struct_field_name: pk_def.field_name.clone(),
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
    let sort_key_def = if let Some(sk_def) = sk_struct_def {
        if let Some(static_value) = sk_def.static_value {
            // Ensure that no other field defs with sk
            if !sk_field_defs.is_empty() {
                let mut diagnostic = syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "If a struct-level #[sk(name = ..., value = ...)] with static value is defined, no other fields can be annotated with #[sk]",
                );

                for sk_field_def in sk_field_defs {
                    diagnostic.combine(syn::Error::new(
                        sk_field_def.span,
                        "This field defines `#[sk]`.",
                    ));
                }

                return Err(diagnostic);
            }

            Some(KeyDef {
                attribute_name: sk_def.name,
                attribute_value: AttributeValue::Static(static_value),
            })
        } else {
            let mut sk_segments: Vec<(Option<usize>, Segment)> = vec![];
            for field_info in &all_field_defs {
                if let RawStructFieldDefs::Sk(RawSkFieldDef {
                    prefix,
                    order,
                    field_name: name,
                    ..
                }) = &field_info
                {
                    sk_segments.push((
                        *order,
                        Segment {
                            struct_field_name: name.clone(),
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
        }
    } else {
        // Make sure there is at most one SK in field infos
        let mut yo: Vec<&RawSkFieldDef> = all_field_defs
            .iter()
            .filter_map(|field| {
                if let RawStructFieldDefs::Sk(a) = &field {
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
                attribute_name: sk_def.field_name.clone(),
                attribute_value: AttributeValue::Composite(CompositeAttributeValue {
                    prefix: sk_def.prefix.clone(),
                    suffix: None,
                    segments: vec![Segment {
                        struct_field_name: sk_def.field_name.clone(),
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
    for nk_struct_def in nk_struct_defs {
        if let Some(v) = nk_struct_def.static_value {
            nk_map.insert(
                nk_struct_def.name.clone(),
                KeyDef {
                    attribute_name: nk_struct_def.name,
                    attribute_value: AttributeValue::Static(v),
                },
            );
        } else {
            nk_map.insert(
                nk_struct_def.name.clone(),
                KeyDef {
                    attribute_name: nk_struct_def.name,
                    attribute_value: AttributeValue::Composite(CompositeAttributeValue {
                        prefix: nk_struct_def.value_prefix,
                        suffix: nk_struct_def.value_suffix,
                        segments: vec![],
                    }),
                },
            );
        }
    }

    // add field-level NKs
    for nk_field_def in &nk_field_defs {
        let RawNkFieldDef {
            field_name,
            name: tied_to,
            prefix,
            ..
        } = &nk_field_def;

        // Hack
        let look_up_key = if tied_to.is_empty() {
            field_name.clone()
        } else {
            tied_to.clone()
        };

        nk_map
            .entry(look_up_key)
            .and_modify(|key_def| {
                if let AttributeValue::Composite(CompositeAttributeValue { segments, .. }) =
                    &mut key_def.attribute_value
                {
                    segments.push(Segment {
                        struct_field_name: field_name.clone(),
                        prefix: prefix.clone(),
                    });
                }
            })
            .or_insert(KeyDef {
                attribute_name: field_name.clone(),
                attribute_value: AttributeValue::Composite(CompositeAttributeValue {
                    prefix: None,
                    suffix: None,
                    segments: vec![Segment {
                        struct_field_name: field_name.clone(),
                        prefix: prefix.clone(),
                    }],
                }),
            });
    }

    // sort NK segments by order and flatten

    let non_key_defs: Vec<KeyDef<AttributeValue>> = nk_map.into_values().collect();

    Ok(SchemaV2 {
        partition_key_def,
        sort_key_def,
        non_key_defs,
    })
}
