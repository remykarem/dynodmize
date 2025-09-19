use serde::Serialize;
use entity_macros::{based_on, Entity1, EntityModel};
use aws_sdk_dynamodb::{Client, Error};

#[derive(Entity1)]
pub struct User {
    #[partition(
        pk_target = "pk",
        pk_attribute_segment_name = None,
        pk_attribute_segment_order = 0,
        serialize_as_non_key = false,
    )]
    last_name: String,

    #[partition(
        pk_target = "pk",
        pk_attribute_segment_name = None,
        pk_attribute_segment_order = 1,
        serialize_as_non_key = false,
    )]
    first_name: String,

    #[sort(
        sk_target = "sk",
        sk_attribute_segment_name = None,
        sk_attribute_segment_order = 0,
        serialize_as_non_key = true,
    )]
    attribute2: String,

    #[sort(
        sk_target = "sk",
        sk_attribute_segment_name = "ATTRIBUTE3",
        sk_attribute_segment_order = 1,
        serialize_as_non_key = false,
    )]
    attribute3: String,

    #[sort(
        sk_target = "sk",
        sk_attribute_segment_name = "ATTRIBUTE4",
        sk_attribute_segment_order = 2,
        serialize_as_non_key = false,
    )]
    attribute4: String,

    attribute5: String,
}


// ── ENTITY ────────────────────────────────────────
#[derive(Debug, Default, EntityModel, Serialize)]
pub struct Entity2 {
    #[partition_key]
    pub pk: String,

    #[sort(key = "dd")]
    pub attribute2_hello: bool,

    #[sort(key = "dd")]
    pub attribute3: String,

    pub attribute4: bool,
}

// ── REPO ──────────────────────────────────────────
#[based_on(Entity2)]
pub struct Entity2Repo;
