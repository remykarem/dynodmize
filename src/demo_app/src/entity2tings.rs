use aws_sdk_dynamodb::{Client, Error};
use entity_macros::{Entity, EntityModel, based_on};
use serde::Serialize;

#[derive(Entity)]
#[pk(name = "last_name")]
pub struct User {
    #[pk(order = 0)]
    last_name: String,

    #[pk(order = 1)]
    first_name: String,

    #[sk(order = 0)]
    attribute2: String,

    #[sk(prefix = "ATTRIBUTE3", order = 1)]
    attribute3: String,

    #[sk(prefix = "ATTRIBUTE4", order = 2)]
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
