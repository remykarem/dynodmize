use entity_core::{AttributeValue, Segment};
use proc_macro2::TokenStream;
use quote::quote;

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
                Some(entity_core::KeyDef {
                    attribute_name: #attribute_name.to_string(),
                    attribute_value: entity_core::AttributeValue::Static(#static_name.to_string()),
                })
            }
        }
        AttributeValue::Composite(composite_attribute_value) => {
            let sk_vp = tok_optional_string(&composite_attribute_value.prefix);
            let sk_vs = tok_optional_string(&composite_attribute_value.suffix);
            let sk_segments = tok_segments(&composite_attribute_value.segments);
            quote! {
                Some(entity_core::KeyDef {
                    attribute_name: #attribute_name.to_string(),
                    attribute_value: entity_core::AttributeValue::Composite(entity_core::CompositeAttributeValue {
                        segments: #sk_segments,
                        prefix: #sk_vp,
                        suffix: #sk_vs,
                    }),
                })
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
