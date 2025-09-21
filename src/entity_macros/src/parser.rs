use crate::{codegen, schema};
use entity_core::SchemaV2;
use proc_macro2::{Span, TokenStream};
use std::collections::HashSet;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Lit, Meta};


pub fn expand_entity(input: &DeriveInput) -> TokenStream {
    match parse_entity(input) {
        Ok(schema) => codegen::generate_impl(input, schema),
        Err(err) => err.to_compile_error(),
    }
}

fn parse_entity(input: &DeriveInput) -> Result<SchemaV2, Error> {
    let (pk_def, sk_def, nk_defs) = parse_entity_attrs(input)?;
    let field_infos = parse_struct_fields(input)?;
    let schema = schema::build_schema(pk_def, sk_def, nk_defs, field_infos)?;
    Ok(schema)
}

//
// ─── ENTITY LEVEL ATTRS ─────────────────────────────────────────────────────────
//

#[derive(Clone)]
pub struct RawPkStructDef {
    pub(crate) name: String,
    pub(crate) value_prefix: Option<String>,
    pub(crate) value_suffix: Option<String>,
}

pub struct RawPkFieldDef {
    pub name: String,
    pub prefix: Option<String>,
    pub order: Option<usize>,
    pub span: Span,
}

pub struct RawSkFieldDef {
    pub name: String,
    pub prefix: Option<String>,
    pub order: Option<usize>,
    pub span: Span,
}

pub struct RawNkFieldDef {
    pub name: String,
    pub prefix: Option<String>,
    pub order: Option<usize>,
    pub span: Span,
}

pub struct RawSkStructDef {
    pub(crate) name: String,
    pub(crate) value_prefix: Option<String>,
    pub(crate) value_suffix: Option<String>,
    value: Option<String>,
}

pub struct NkDef {
    pub(crate) name: String,
    pub(crate) value_prefix: Option<String>,
    pub(crate) value_suffix: Option<String>,
    pub(crate) static_value: Option<String>,
}

pub struct RawStructFieldDefs {
    pub(crate) field_name: String,
    pub(crate) raw_field_def: RawFieldDef,
}

pub enum RawFieldDef {
    Pk(RawPkFieldDef),
    Sk(RawSkFieldDef),
    Nk(RawNkFieldDef),
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
        } else {
            return Err(Error::new_spanned(attr, "Expected #[pk(...)] or #[sk(...)]"));
        }
    }

    Ok((pk, sk, nks))
}

//
// ─── FIELD LEVEL ATTRS ──────────────────────────────────────────────────────────
//

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
                            name: name.clone().unwrap_or_else(|| ident.to_string()),
                            prefix: prefix.clone(),
                            order,
                            span: list.span(),
                        })
                    }

                    if attr.path().is_ident("sk") {
                        sk_defs.push(RawSkFieldDef {
                            name: name.clone().unwrap_or_else(|| ident.to_string()),
                            prefix: prefix.clone(),
                            order,
                            span: list.span(),
                        })
                    }

                    if attr.path().is_ident("nk") {
                        let name = name
                            .ok_or_else(|| Error::new_spanned(attr, "nk field must have name"))?;
                        nk_defs.push(RawNkFieldDef {
                            name: name.clone(),
                            prefix,
                            order,
                            span: list.span(),
                        });
                    }
                }
                // #[pk]
                Meta::Path(path) => {
                    if attr.path().is_ident("pk") {
                        pk_defs.push(RawPkFieldDef {
                            name: ident.to_string(),
                            prefix: None,
                            order,
                            span: attr.meta.span(),
                        })
                    }

                    if attr.path().is_ident("sk") {
                        sk_defs.push(RawSkFieldDef {
                            name: name.clone().unwrap_or_else(|| ident.to_string()),
                            prefix: None,
                            order,
                            span: path.span(),
                        })
                    }

                    if attr.path().is_ident("nk") {
                        nk_defs.push(RawNkFieldDef {
                            name: ident.to_string(),
                            prefix: None,
                            order: None,
                            span: path.span(),
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
                "Cannot have more than 1 #[pk] per field",
            ));
        }
        if sk_defs.len() > 1 {
            return Err(Error::new_spanned(
                field,
                "Cannot have more than 1 #[sk] per field",
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

        // If multiple pks are defined, check if all of them have order

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
