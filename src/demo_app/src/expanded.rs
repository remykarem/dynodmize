#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
mod entity2tings {
    use aws_sdk_dynamodb::Client;
    use entity_macros::{based_on, Entity, EntityModel};
    use serde::Serialize;
    #[pk(name = "mypk")]
    pub struct ComplaintComments {
        pub complaint_id: u32,
        #[pk(order = 1, prefix = "COMMENT_ID")]
        pub comment_id: u32,
        #[sk]
        pub comment_date: String,
        pub comment_dates: String,
        pub attribute2: String,
    }
    impl entity_core::Entity2 for ComplaintComments {
        fn get_schema() -> entity_core::SchemaV2 {
            let partition_key_def = entity_core::KeyDef {
                attribute_name: "mypk".to_string(),
                attribute_value: entity_core::CompositeAttributeValue {
                    segments: Vec::<
                        entity_core::Segment,
                    >::from([
                        entity_core::Segment {
                            struct_field_name: "comment_id".to_string(),
                            prefix: Some("COMMENT_ID".to_string()),
                        },
                    ]),
                    prefix: None,
                    suffix: None,
                },
            };
            let sort_key_def = None;
            let non_key_defs: Vec<entity_core::KeyDef<entity_core::AttributeValue>> = Vec::<
                entity_core::KeyDef<entity_core::AttributeValue>,
            >::from([]);
            entity_core::SchemaV2 {
                partition_key_def,
                sort_key_def,
                non_key_defs,
            }
        }
        fn to_item(&self) -> serde_json::Value {
            let mut map = serde_json::Map::new();
            map.insert(
                "mypk".to_string(),
                serde_json::Value::String(
                    Vec::<
                        String,
                    >::from([
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("{0}#{1}", "COMMENT_ID", self.comment_id),
                                )
                            }),
                        ])
                        .join("#"),
                ),
            );
            let sk_expr = { ::std::collections::HashMap::<String, String>::new() };
            if let Some((attr_name, sk_val)) = sk_expr {
                map.insert(attr_name, serde_json::Value::String(sk_val));
            }
            serde_json::Value::Object(map)
        }
    }
    #[pk(name = "last_name")]
    #[sk(name = "dd")]
    #[nk(name = "type", value = "dynamo")]
    pub struct User {
        #[pk(order = 0, prefix = "ATTR2")]
        pub attribute2: String,
        pub last_name: String,
        #[pk(order = 1, prefix = "FIRSTNAME")]
        pub first_name: String,
        #[sk(order = 1, prefix = "ATTR3")]
        pub attribute3: String,
        #[sk(order = 0)]
        pub attribute4: String,
        pub attribute5: String,
    }
    impl entity_core::Entity2 for User {
        fn get_schema() -> entity_core::SchemaV2 {
            let partition_key_def = entity_core::KeyDef {
                attribute_name: "last_name".to_string(),
                attribute_value: entity_core::CompositeAttributeValue {
                    segments: Vec::<
                        entity_core::Segment,
                    >::from([
                        entity_core::Segment {
                            struct_field_name: "attribute2".to_string(),
                            prefix: Some("ATTR2".to_string()),
                        },
                        entity_core::Segment {
                            struct_field_name: "first_name".to_string(),
                            prefix: Some("FIRSTNAME".to_string()),
                        },
                    ]),
                    prefix: None,
                    suffix: None,
                },
            };
            let sort_key_def = Some(entity_core::KeyDef {
                attribute_name: "dd".to_string(),
                attribute_value: entity_core::AttributeValue::Composite(entity_core::CompositeAttributeValue {
                    segments: Vec::<
                        entity_core::Segment,
                    >::from([
                        entity_core::Segment {
                            struct_field_name: "attribute4".to_string(),
                            prefix: None,
                        },
                        entity_core::Segment {
                            struct_field_name: "attribute3".to_string(),
                            prefix: Some("ATTR3".to_string()),
                        },
                    ]),
                    prefix: None,
                    suffix: None,
                }),
            });
            let non_key_defs: Vec<entity_core::KeyDef<entity_core::AttributeValue>> = Vec::<
                entity_core::KeyDef<entity_core::AttributeValue>,
            >::from([
                entity_core::KeyDef {
                    attribute_name: "type".to_string(),
                    attribute_value: entity_core::AttributeValue::Static(
                        "dynamo".to_string(),
                    ),
                },
            ]);
            entity_core::SchemaV2 {
                partition_key_def,
                sort_key_def,
                non_key_defs,
            }
        }
        fn to_item(&self) -> serde_json::Value {
            let mut map = serde_json::Map::new();
            map.insert(
                "last_name".to_string(),
                serde_json::Value::String(
                    Vec::<
                        String,
                    >::from([
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("{0}#{1}", "ATTR2", self.attribute2),
                                )
                            }),
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("{0}#{1}", "FIRSTNAME", self.first_name),
                                )
                            }),
                        ])
                        .join("#"),
                ),
            );
            let sk_expr = {
                let mut map: ::std::collections::HashMap<String, String> = ::std::collections::HashMap::new();
                let mut parts: Vec<String> = Vec::new();
                parts
                    .extend(
                        <[_]>::into_vec(
                            ::alloc::boxed::box_new([
                                self.attribute4.to_string(),
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("{0}#{1}", "ATTR3", self.attribute3),
                                    )
                                }),
                            ]),
                        ),
                    );
                let composite_sk = parts.join("#");
                map.insert("dd".to_string(), composite_sk);
                map
            };
            if let Some((attr_name, sk_val)) = sk_expr {
                map.insert(attr_name, serde_json::Value::String(sk_val));
            }
            map.insert(
                "type".to_string(),
                serde_json::Value::String("dynamo".to_string()),
            );
            serde_json::Value::Object(map)
        }
    }
    pub struct MyEntity2 {
        #[partition_key]
        pub pk: String,
        #[sort(key = "dd")]
        pub attribute2_hello: bool,
        #[sort(key = "dd")]
        pub attribute3: String,
        pub attribute4: bool,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for MyEntity2 {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "MyEntity2",
                "pk",
                &self.pk,
                "attribute2_hello",
                &self.attribute2_hello,
                "attribute3",
                &self.attribute3,
                "attribute4",
                &&self.attribute4,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for MyEntity2 {
        #[inline]
        fn default() -> MyEntity2 {
            MyEntity2 {
                pk: ::core::default::Default::default(),
                attribute2_hello: ::core::default::Default::default(),
                attribute3: ::core::default::Default::default(),
                attribute4: ::core::default::Default::default(),
            }
        }
    }
    impl entity_core::Entity for MyEntity2 {
        fn get_partition_key(&self) -> String {
            self.pk.clone()
        }
        fn get_sort_key(&self) -> Option<String> {
            Some(
                <[_]>::into_vec(
                        ::alloc::boxed::box_new([
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "{0}#{1}",
                                        "ATTRIBUTE2_HELLO",
                                        self.attribute2_hello,
                                    ),
                                )
                            }),
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("{0}#{1}", "ATTRIBUTE3", self.attribute3),
                                )
                            }),
                        ]),
                    )
                    .join("#"),
            )
        }
    }
    pub trait MyEntity2Setters: entity_core::HasInner<MyEntity2> + Sized {
        fn set_attribute2_hello(self, value: bool) -> Self;
        fn set_attribute3(self, value: String) -> Self;
        fn set_attribute4(self, value: bool) -> Self;
    }
    impl MyEntity2Setters for entity_core::UpdateBuilderWithSetters<MyEntity2> {
        fn set_attribute2_hello(mut self, value: bool) -> Self {
            let v = value.clone();
            self.inner_mut()
                .updates
                .push(
                    Box::new(move |e: &mut MyEntity2| {
                        e.attribute2_hello = v.clone();
                    }),
                );
            self
        }
        fn set_attribute3(mut self, value: String) -> Self {
            let v = value.clone();
            self.inner_mut()
                .updates
                .push(
                    Box::new(move |e: &mut MyEntity2| {
                        e.attribute3 = v.clone();
                    }),
                );
            self
        }
        fn set_attribute4(mut self, value: bool) -> Self {
            let v = value.clone();
            self.inner_mut()
                .updates
                .push(
                    Box::new(move |e: &mut MyEntity2| {
                        e.attribute4 = v.clone();
                    }),
                );
            self
        }
    }
    impl entity_core::HasInner<MyEntity2>
    for entity_core::UpdateBuilderWithSetters<MyEntity2> {
        fn inner_mut(&mut self) -> &mut entity_core::UpdateBuilder<MyEntity2> {
            &mut self.inner
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for MyEntity2 {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private225::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "MyEntity2",
                    false as usize + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "pk",
                    &self.pk,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "attribute2_hello",
                    &self.attribute2_hello,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "attribute3",
                    &self.attribute3,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "attribute4",
                    &self.attribute4,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    use entity_core::UpdateBuilderWithSetters;
    pub struct Entity2Repo;
    impl Entity2Repo {
        /// Hello
        pub fn create(
            &self,
            entity: MyEntity2,
            client: Client,
        ) -> entity_core::CreateBuilder<MyEntity2> {
            entity_core::CreateBuilder {
                entity,
                client,
            }
        }
        pub fn query(&self) -> entity_core::QueryBuilder<MyEntity2> {
            entity_core::QueryBuilder {
                partition_key: None,
                _marker: std::marker::PhantomData,
            }
        }
        pub fn update(&self) -> UpdateBuilderWithSetters<MyEntity2> {
            UpdateBuilderWithSetters {
                inner: entity_core::UpdateBuilder {
                    partition_key: None,
                    updates: ::alloc::vec::Vec::new(),
                },
            }
        }
    }
}
mod expanded {
    #![feature(prelude_import)]
    #[prelude_import]
    use std::prelude::rust_2024::*;
    #[macro_use]
    extern crate std;
    mod entity2tings {
        use aws_sdk_dynamodb::Client;
        use entity_macros::{based_on, Entity, EntityModel};
        use serde::Serialize;
        #[pk(name = "mypk")]
        pub struct ComplaintComments {
            pub complaint_id: u32,
            #[pk(order = 1, prefix = "COMMENT_ID")]
            pub comment_id: u32,
            #[sk]
            pub comment_date: String,
            pub comment_dates: String,
            pub attribute2: String,
        }
        impl entity_core::Entity2 for ComplaintComments {
            fn get_schema() -> entity_core::SchemaV2 {
                let partition_key_def = entity_core::KeyDef {
                    attribute_name: "mypk".to_string(),
                    attribute_value: entity_core::CompositeAttributeValue {
                        segments: Vec::<
                            entity_core::Segment,
                        >::from([
                            entity_core::Segment {
                                struct_field_name: "comment_id".to_string(),
                                prefix: Some("COMMENT_ID".to_string()),
                            },
                        ]),
                        prefix: None,
                        suffix: None,
                    },
                };
                let sort_key_def = None;
                let non_key_defs: Vec<
                    entity_core::KeyDef<entity_core::AttributeValue>,
                > = Vec::<entity_core::KeyDef<entity_core::AttributeValue>>::from([]);
                entity_core::SchemaV2 {
                    partition_key_def,
                    sort_key_def,
                    non_key_defs,
                }
            }
            fn to_item(&self) -> serde_json::Value {
                let mut map = serde_json::Map::new();
                map.insert(
                    "mypk".to_string(),
                    serde_json::Value::String(
                        Vec::<
                            String,
                        >::from([
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("{0}#{1}", "COMMENT_ID", self.comment_id),
                                    )
                                }),
                            ])
                            .join("#"),
                    ),
                );
                let sk_expr = { ::std::collections::HashMap::<String, String>::new() };
                if let Some((attr_name, sk_val)) = sk_expr {
                    map.insert(attr_name, serde_json::Value::String(sk_val));
                }
                serde_json::Value::Object(map)
            }
        }
        #[pk(name = "last_name")]
        #[sk(name = "dd")]
        #[nk(name = "type", value = "dynamo")]
        pub struct User {
            #[pk(order = 0, prefix = "ATTR2")]
            pub attribute2: String,
            pub last_name: String,
            #[pk(order = 1, prefix = "FIRSTNAME")]
            pub first_name: String,
            #[sk(order = 1, prefix = "ATTR3")]
            pub attribute3: String,
            #[sk(order = 0)]
            pub attribute4: String,
            pub attribute5: String,
        }
        impl entity_core::Entity2 for User {
            fn get_schema() -> entity_core::SchemaV2 {
                let partition_key_def = entity_core::KeyDef {
                    attribute_name: "last_name".to_string(),
                    attribute_value: entity_core::CompositeAttributeValue {
                        segments: Vec::<
                            entity_core::Segment,
                        >::from([
                            entity_core::Segment {
                                struct_field_name: "attribute2".to_string(),
                                prefix: Some("ATTR2".to_string()),
                            },
                            entity_core::Segment {
                                struct_field_name: "first_name".to_string(),
                                prefix: Some("FIRSTNAME".to_string()),
                            },
                        ]),
                        prefix: None,
                        suffix: None,
                    },
                };
                let sort_key_def = Some(entity_core::KeyDef {
                    attribute_name: "dd".to_string(),
                    attribute_value: entity_core::AttributeValue::Composite(entity_core::CompositeAttributeValue {
                        segments: Vec::<
                            entity_core::Segment,
                        >::from([
                            entity_core::Segment {
                                struct_field_name: "attribute4".to_string(),
                                prefix: None,
                            },
                            entity_core::Segment {
                                struct_field_name: "attribute3".to_string(),
                                prefix: Some("ATTR3".to_string()),
                            },
                        ]),
                        prefix: None,
                        suffix: None,
                    }),
                });
                let non_key_defs: Vec<
                    entity_core::KeyDef<entity_core::AttributeValue>,
                > = Vec::<
                    entity_core::KeyDef<entity_core::AttributeValue>,
                >::from([
                    entity_core::KeyDef {
                        attribute_name: "type".to_string(),
                        attribute_value: entity_core::AttributeValue::Static(
                            "dynamo".to_string(),
                        ),
                    },
                ]);
                entity_core::SchemaV2 {
                    partition_key_def,
                    sort_key_def,
                    non_key_defs,
                }
            }
            fn to_item(&self) -> serde_json::Value {
                let mut map = serde_json::Map::new();
                map.insert(
                    "last_name".to_string(),
                    serde_json::Value::String(
                        Vec::<
                            String,
                        >::from([
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("{0}#{1}", "ATTR2", self.attribute2),
                                    )
                                }),
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("{0}#{1}", "FIRSTNAME", self.first_name),
                                    )
                                }),
                            ])
                            .join("#"),
                    ),
                );
                let sk_expr = {
                    let mut map: ::std::collections::HashMap<String, String> = ::std::collections::HashMap::new();
                    let composite_sk_parts: Vec<_> = <[_]>::into_vec(
                        ::alloc::boxed::box_new([
                            self.attribute4.to_string(),
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!("{0}#{1}", "ATTR3", self.attribute3),
                                )
                            }),
                        ]),
                    );
                    let composite_sk = composite_sk_parts.join("#");
                    map.insert("dd".to_string(), composite_sk);
                    map.insert(attribute4.to_string(), self.attribute4.to_string());
                    map.insert(attribute3.to_string(), self.attribute3.to_string());
                    map
                };
                if let Some((attr_name, sk_val)) = sk_expr {
                    map.insert(attr_name, serde_json::Value::String(sk_val));
                }
                map.insert(
                    "type".to_string(),
                    serde_json::Value::String("dynamo".to_string()),
                );
                serde_json::Value::Object(map)
            }
        }
        pub struct MyEntity2 {
            #[partition_key]
            pub pk: String,
            #[sort(key = "dd")]
            pub attribute2_hello: bool,
            #[sort(key = "dd")]
            pub attribute3: String,
            pub attribute4: bool,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for MyEntity2 {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field4_finish(
                    f,
                    "MyEntity2",
                    "pk",
                    &self.pk,
                    "attribute2_hello",
                    &self.attribute2_hello,
                    "attribute3",
                    &self.attribute3,
                    "attribute4",
                    &&self.attribute4,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for MyEntity2 {
            #[inline]
            fn default() -> MyEntity2 {
                MyEntity2 {
                    pk: ::core::default::Default::default(),
                    attribute2_hello: ::core::default::Default::default(),
                    attribute3: ::core::default::Default::default(),
                    attribute4: ::core::default::Default::default(),
                }
            }
        }
        impl entity_core::Entity for MyEntity2 {
            fn get_partition_key(&self) -> String {
                self.pk.clone()
            }
            fn get_sort_key(&self) -> Option<String> {
                Some(
                    <[_]>::into_vec(
                            ::alloc::boxed::box_new([
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "{0}#{1}",
                                            "ATTRIBUTE2_HELLO",
                                            self.attribute2_hello,
                                        ),
                                    )
                                }),
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("{0}#{1}", "ATTRIBUTE3", self.attribute3),
                                    )
                                }),
                            ]),
                        )
                        .join("#"),
                )
            }
        }
        pub trait MyEntity2Setters: entity_core::HasInner<MyEntity2> + Sized {
            fn set_attribute2_hello(self, value: bool) -> Self;
            fn set_attribute3(self, value: String) -> Self;
            fn set_attribute4(self, value: bool) -> Self;
        }
        impl MyEntity2Setters for entity_core::UpdateBuilderWithSetters<MyEntity2> {
            fn set_attribute2_hello(mut self, value: bool) -> Self {
                let v = value.clone();
                self.inner_mut()
                    .updates
                    .push(
                        Box::new(move |e: &mut MyEntity2| {
                            e.attribute2_hello = v.clone();
                        }),
                    );
                self
            }
            fn set_attribute3(mut self, value: String) -> Self {
                let v = value.clone();
                self.inner_mut()
                    .updates
                    .push(
                        Box::new(move |e: &mut MyEntity2| {
                            e.attribute3 = v.clone();
                        }),
                    );
                self
            }
            fn set_attribute4(mut self, value: bool) -> Self {
                let v = value.clone();
                self.inner_mut()
                    .updates
                    .push(
                        Box::new(move |e: &mut MyEntity2| {
                            e.attribute4 = v.clone();
                        }),
                    );
                self
            }
        }
        impl entity_core::HasInner<MyEntity2>
        for entity_core::UpdateBuilderWithSetters<MyEntity2> {
            fn inner_mut(&mut self) -> &mut entity_core::UpdateBuilder<MyEntity2> {
                &mut self.inner
            }
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for MyEntity2 {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private225::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "MyEntity2",
                        false as usize + 1 + 1 + 1 + 1,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "pk",
                        &self.pk,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "attribute2_hello",
                        &self.attribute2_hello,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "attribute3",
                        &self.attribute3,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "attribute4",
                        &self.attribute4,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        use entity_core::UpdateBuilderWithSetters;
        pub struct Entity2Repo;
        impl Entity2Repo {
            /// Hello
            pub fn create(
                &self,
                entity: MyEntity2,
                client: Client,
            ) -> entity_core::CreateBuilder<MyEntity2> {
                entity_core::CreateBuilder {
                    entity,
                    client,
                }
            }
            pub fn query(&self) -> entity_core::QueryBuilder<MyEntity2> {
                entity_core::QueryBuilder {
                    partition_key: None,
                    _marker: std::marker::PhantomData,
                }
            }
            pub fn update(&self) -> UpdateBuilderWithSetters<MyEntity2> {
                UpdateBuilderWithSetters {
                    inner: entity_core::UpdateBuilder {
                        partition_key: None,
                        updates: ::alloc::vec::Vec::new(),
                    },
                }
            }
        }
    }
    use crate::entity2tings::{
        ComplaintComments, Entity2Repo, MyEntity2, MyEntity2Setters,
    };
    use aws_config::meta::region::RegionProviderChain;
    use aws_sdk_dynamodb::config::{BehaviorVersion, Region};
    use aws_sdk_dynamodb::Client;
    use entity_core::Entity2;
    use entity_core::*;
    fn main() {
        {
            ::std::io::_print(format_args!("{0:?}\n", ComplaintComments::get_schema()));
        };
        let ent = ComplaintComments {
            complaint_id: 123,
            comment_date: "somedate".to_string(),
            comment_dates: "somedatess".to_string(),
            comment_id: 456,
            attribute2: "d".to_string(),
        };
        {
            ::std::io::_print(format_args!("{0}\n", ent.to_item()));
        };
    }
    #[allow(dead_code)]
    async fn main2() {
        let region_provider = RegionProviderChain::default_provider()
            .or_else(Region::new("ap-southeast-1"));
        let shared_config = aws_config::defaults(BehaviorVersion::v2025_08_07())
            .region(region_provider)
            .load()
            .await;
        let client = Client::new(&shared_config);
        let m = client.list_tables().send().await.unwrap();
        {
            ::std::io::_print(format_args!("{0:?}\n", m.table_names));
        };
        let repo = Entity2Repo;
        let entity = MyEntity2 {
            pk: "pk_123".into(),
            attribute2_hello: true,
            attribute3: "sk_partB".into(),
            attribute4: true,
        };
        {
            ::std::io::_print(format_args!("PK: {0}\n", entity.get_partition_key()));
        };
        {
            ::std::io::_print(format_args!("SK: {0}\n", entity.get_sort_key().unwrap()));
        };
        repo.create(entity, client).send2().await.unwrap();
        let results = repo.query().where_partition_key("pk_123").send();
        {
            ::std::io::_print(format_args!("Queried result: {0:?}\n", results));
        };
        repo.update()
            .set_attribute2_hello(true)
            .set_attribute4(false)
            .where_partition_key("pk_123")
            .send();
    }
}
use crate::entity2tings::{ComplaintComments, Entity2Repo, MyEntity2, MyEntity2Setters};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::config::{BehaviorVersion, Region};
use aws_sdk_dynamodb::Client;
use entity_core::Entity2;
use entity_core::*;
fn main() {
    {
        ::std::io::_print(format_args!("{0:?}\n", ComplaintComments::get_schema()));
    };
    let ent = ComplaintComments {
        complaint_id: 123,
        comment_date: "somedate".to_string(),
        comment_dates: "somedatess".to_string(),
        comment_id: 456,
        attribute2: "d".to_string(),
    };
    {
        ::std::io::_print(format_args!("{0}\n", ent.to_item()));
    };
}
#[allow(dead_code)]
async fn main2() {
    let region_provider = RegionProviderChain::default_provider()
        .or_else(Region::new("ap-southeast-1"));
    let shared_config = aws_config::defaults(BehaviorVersion::v2025_08_07())
        .region(region_provider)
        .load()
        .await;
    let client = Client::new(&shared_config);
    let m = client.list_tables().send().await.unwrap();
    {
        ::std::io::_print(format_args!("{0:?}\n", m.table_names));
    };
    let repo = Entity2Repo;
    let entity = MyEntity2 {
        pk: "pk_123".into(),
        attribute2_hello: true,
        attribute3: "sk_partB".into(),
        attribute4: true,
    };
    {
        ::std::io::_print(format_args!("PK: {0}\n", entity.get_partition_key()));
    };
    {
        ::std::io::_print(format_args!("SK: {0}\n", entity.get_sort_key().unwrap()));
    };
    repo.create(entity, client).send2().await.unwrap();
    let results = repo.query().where_partition_key("pk_123").send();
    {
        ::std::io::_print(format_args!("Queried result: {0:?}\n", results));
    };
    repo.update()
        .set_attribute2_hello(true)
        .set_attribute4(false)
        .where_partition_key("pk_123")
        .send();
}
