use aws_sdk_dynamodb::{Client, Error};
use serde::Serialize;
use serde_dynamo::to_item;
use std::fmt::Debug;
use std::collections::HashSet;
use serde_json::Value;

//
// ─── ENTITY TRAIT ───────────────────────────────────────────────────────────────
//
pub trait Entity {
    fn get_partition_key(&self) -> String;
    fn get_sort_key(&self) -> Option<String> {
        None
    }
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



pub trait Entity1 {
    fn get_schema() -> Schema;
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub partition_key: CompositeKey,
    pub sort_key: Option<CompositeKey>,
    pub data: HashSet<Attribute>,
    pub delimiter: char,
}

#[derive(Debug, Clone)]
pub struct CompositeKey {
    pub attribute_name: String,                 // e.g. "pk", "sk"
    pub attributes: Vec<CompositeKeyAttribute>, // segments
}

#[derive(Debug, Clone)]
pub struct CompositeKeyAttribute {
    pub attribute_segment_name_in_key: Option<String>, // e.g. "ATTRIBUTE3"
    pub attribute_name_in_data: String,                // e.g. "attribute3"
    pub serialize_value_in_data: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Attribute {
    pub name: String,
}