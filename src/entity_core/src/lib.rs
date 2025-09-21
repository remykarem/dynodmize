use aws_sdk_dynamodb::{Client, Error};
use serde::Serialize;
use serde_dynamo::to_item;
use std::fmt::Debug;

//
// ─── ENTITY TRAIT ───────────────────────────────────────────────────────────────
//
pub trait Dynodmize {
    fn get_partition_key(&self) -> String;
    fn get_sort_key(&self) -> Option<String> {
        None
    }
}
pub trait Entity2 {
    fn get_schema() -> SchemaV2;
    fn to_item(&self) -> serde_json::Value;
}

//
// ─── CREATE BUILDER ─────────────────────────────────────────────────────────────
//
pub struct CreateBuilder<T> {
    pub entity: T,
    pub client: Client,
}

impl<T: Debug + Serialize> CreateBuilder<T> {
    pub fn send(self) {
        println!("Creating entity: {:?}", self.entity);
    }

    pub async fn send2(self) -> Result<(), Error> {
        let item = Some(to_item(self.entity).unwrap());
        self.client
            .put_item()
            .table_name("test")
            .set_item(item)
            .send()
            .await?;
        Ok(())
    }
}

//
// ─── QUERY BUILDER ──────────────────────────────────────────────────────────────
//
pub struct QueryBuilder<T> {
    pub partition_key: Option<String>,
    pub _marker: std::marker::PhantomData<T>,
}

impl<T: Debug + Default> QueryBuilder<T> {
    pub fn where_partition_key(mut self, key: &str) -> Self {
        self.partition_key = Some(key.to_owned());
        self
    }

    pub fn send(self) -> Vec<T> {
        println!("Query on pk={:?}", self.partition_key);
        vec![T::default()]
    }
}

//
// ─── UPDATE BUILDER ─────────────────────────────────────────────────────────────
//
pub struct UpdateBuilder<T> {
    pub partition_key: Option<String>,
    pub updates: Vec<Box<dyn Fn(&mut T) + 'static>>,
}

impl<T: Debug + Default> UpdateBuilder<T> {
    pub fn where_partition_key(mut self, key: &str) -> Self {
        self.partition_key = Some(key.to_owned());
        self
    }

    pub fn send(self) {
        println!(
            "Update entity at pk={:?} with {} update(s)",
            self.partition_key,
            self.updates.len()
        );
        let mut entity = T::default();
        for f in self.updates {
            f(&mut entity);
        }
        println!("Updated entity would be: {:?}", entity);
    }
}

//
// ─── UPDATE BUILDER WITH SETTERS ────────────────────────────────────────────────
//
pub struct UpdateBuilderWithSetters<T> {
    pub inner: UpdateBuilder<T>,
}

impl<T> UpdateBuilderWithSetters<T> {
    pub fn inner_mut(&mut self) -> &mut UpdateBuilder<T> {
        &mut self.inner
    }
}

//
// ─── TRAIT USED BY MACROS TO AVOID ORPHAN RULE ──────────────────────────────────
//
pub trait HasInner<T> {
    fn inner_mut(&mut self) -> &mut UpdateBuilder<T>;
}

impl<T: Debug + Default> UpdateBuilderWithSetters<T> {
    // by-value -> returns Self (so you can chain on rvalues)
    pub fn where_partition_key(mut self, key: &str) -> Self {
        self.inner = self.inner.where_partition_key(key);
        self
    }

    // by-value -> consumes at the end of the chain
    pub fn send(self) {
        self.inner.send();
    }
}

#[derive(Debug)]
pub struct SchemaV2 {
    pub partition_key_def: KeyDef<CompositeAttributeValue>,
    pub sort_key_def: Option<KeyDef<AttributeValue>>,
    pub non_key_defs: Vec<KeyDef<AttributeValue>>,
}

#[derive(Debug)]
pub struct KeyDef<V> {
    pub attribute_name: String,
    pub attribute_value: V,
}

#[derive(Debug)]
pub enum AttributeValue {
    Static(String),
    Composite(CompositeAttributeValue),
}

#[derive(Debug)]
pub struct CompositeAttributeValue {
    pub segments: Vec<Segment>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

#[derive(Debug)]
pub struct Segment {
    pub struct_field_name: String,
    pub prefix: Option<String>,
}
