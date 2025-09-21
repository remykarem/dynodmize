use std::alloc;
use aws_sdk_dynamodb::{Client, Error};
use entity_macros::{Entity, EntityModel, based_on};
use serde::Serialize;

#[derive(Entity)]
#[sk(
    name = "sk",
)]
#[pk(name = "hello")]
pub struct ComplaintComments {

    #[pk]
    pub complaint_id: u32,

    #[sk(prefix = "COMMENT_ID", order = 1)]
    pub comment_id: u32,

    #[sk(prefix = "COMMENT_DATE", order = 2)]
    pub comment_date: String,

    #[pk(prefix = "dd")]
    pub comment_dates: String,

    pub attribute2: String,
}


#[derive(Entity)]
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

    // #[nk]
    pub attribute5: String,
}

// ── ENTITY ────────────────────────────────────────
#[derive(Debug, Default, EntityModel, Serialize)]
pub struct MyEntity2 {
    #[partition_key]
    pub pk: String,

    #[sort(key = "dd")]
    pub attribute2_hello: bool,

    #[sort(key = "dd")]
    pub attribute3: String,

    pub attribute4: bool,
}

// ── REPO ──────────────────────────────────────────
#[based_on(MyEntity2)]
pub struct Entity2Repo;
