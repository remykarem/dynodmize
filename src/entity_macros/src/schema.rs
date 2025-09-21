use crate::parser::{
    NkDef, RawFieldDef, RawNkFieldDef, RawPkFieldDef, RawPkStructDef, RawSkFieldDef,
    RawSkStructDef, RawStructFieldDefs,
};
use entity_core::{AttributeValue, CompositeAttributeValue, KeyDef, SchemaV2, Segment};
use std::collections::HashMap;
use syn::Error;

pub fn build_ir(
    pk_struct_def: Option<RawPkStructDef>,
    sk_struct_def: Option<RawSkStructDef>,
    nk_defs: Vec<NkDef>,
    field_defs: Vec<RawStructFieldDefs>,
) -> Result<SchemaV2, syn::Error> {
    //
    // ─── BUILD PK ────────────────────────────────────────────────────────────────
    //
    let pk_def = if let Some(pk_def) = pk_struct_def.clone() {
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

    let mut pk_segments: Vec<(Option<usize>, Segment)> = vec![];
    for field_def in &field_defs {
        if let RawFieldDef::Pk(RawPkFieldDef {
            name,
            prefix,
            order,
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
    let pk_segments: Vec<Segment> = pk_segments.into_iter().map(|(_, seg)| seg).collect();

    let pk = KeyDef {
        attribute_name: pk_struct_def.clone().unwrap().name.clone(),
        attribute_value: CompositeAttributeValue {
            prefix: pk_struct_def.clone().unwrap().value_prefix.clone(),
            suffix: pk_struct_def.unwrap().value_suffix,
            segments: pk_segments,
        },
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

pub fn validate_schema(schema: &SchemaV2) -> Result<(), syn::Error> {
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
