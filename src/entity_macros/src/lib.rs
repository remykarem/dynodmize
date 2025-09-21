mod parser;
mod codegen;
mod schema;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemStruct, parse_macro_input};

const PARTITION: &str = "partition_key";
const SORT: &str = "sort";

#[proc_macro_derive(Entity, attributes(pk, sk, nk))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    parser::expand_entity(&input).into()
}

#[proc_macro_derive(EntityModel, attributes(partition_key, sort))]
pub fn derive_entity_model(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input);
    let name = &ast.ident;

    let mut pk_field = None;
    let mut sort_fields = Vec::new();
    let mut signatures = Vec::new();
    let mut impls = Vec::new();

    if let syn::Data::Struct(ds) = &ast.data {
        for field in &ds.fields {
            let ident = field.ident.as_ref().expect("expected named fields");
            let typ = &field.ty;

            let mut is_pk = false;
            for attribute in &field.attrs {
                if attribute.path().is_ident(PARTITION) {
                    pk_field = Some(ident.clone());
                    is_pk = true;
                }
                if attribute.path().is_ident(SORT) {
                    sort_fields.push(ident.clone());
                }
            }

            // Generate setters for all non-PK fields
            if !is_pk {
                let method_name = syn::Ident::new(&format!("set_{}", ident), ident.span());

                // Trait method signature (owned-builder style)
                signatures.push(quote! {
                    fn #method_name(self, value: #typ) -> Self;
                });

                // Impl: mutate inner, return Self
                impls.push(quote! {
                    fn #method_name(mut self, value: #typ) -> Self {
                        let v = value.clone();
                        self.inner_mut().updates.push(Box::new(move |e: &mut #name| {
                            e.#ident = v.clone();
                        }));
                        self
                    }
                });
            }
        }
    }

    let pk_field = pk_field.unwrap_or_else(|| panic!("No `#[{PARTITION}]` field found"));

    // Per-entity trait name, e.g. Entity2Setters
    let setters_trait = syn::Ident::new(&format!("{}Setters", name), name.span());

    // Build sort key join expression from all #[sort] fields in declaration order
    let mut parts = Vec::new();
    for f in &sort_fields {
        let upper = f.to_string().to_uppercase();
        parts.push(quote! {
            format!("{}#{}", #upper, self.#f)
        });
    }

    let sort_key_fn = if parts.is_empty() {
        quote! { None }
    } else {
        quote! { Some(vec![#(#parts),*].join("#")) }
    };

    let expanded = quote! {
        // Implement core Entity methods
        impl entity_core::Entity for #name {
            fn get_partition_key(&self) -> String {
                self.#pk_field.clone()
            }
            fn get_sort_key(&self) -> Option<String> {
                #sort_key_fn
            }
        }

        // Per-entity setters trait; owned-builder style (Self by value)
        pub trait #setters_trait: entity_core::HasInner<#name> + Sized {
            #(#signatures)*
        }

        // Implement the setters for the outer builder wrapper
        impl #setters_trait for entity_core::UpdateBuilderWithSetters<#name> {
            #(#impls)*
        }

        // Hook the wrapper into HasInner so setters can reach the inner builder
        impl entity_core::HasInner<#name> for entity_core::UpdateBuilderWithSetters<#name> {
            fn inner_mut(&mut self) -> &mut entity_core::UpdateBuilder<#name> {
                &mut self.inner
            }
        }
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn based_on(args: TokenStream, input: TokenStream) -> TokenStream {
    let entity_ty: syn::Type = syn::parse(args).expect("expected a type in #[based_on(..)]");
    let repo_struct: ItemStruct = syn::parse(input).expect("expected a struct after #[based_on]");
    let repo_name = &repo_struct.ident;

    let expanded = quote! {
        use entity_core::UpdateBuilderWithSetters;

        #repo_struct

        impl #repo_name {
            /// Hello
            pub fn create(&self, entity: #entity_ty, client: Client)
                -> entity_core::CreateBuilder<#entity_ty>
            {
                entity_core::CreateBuilder { entity, client }
            }

            pub fn query(&self) -> entity_core::QueryBuilder<#entity_ty> {
                entity_core::QueryBuilder {
                    partition_key: None,
                    _marker: std::marker::PhantomData,
                }
            }

            pub fn update(&self) -> UpdateBuilderWithSetters<#entity_ty> {
                UpdateBuilderWithSetters {
                    inner: entity_core::UpdateBuilder {
                        partition_key: None,
                        updates: vec![],
                    }
                }
            }
        }
    };

    expanded.into()
}
