mod entity2tings;

use crate::entity2tings::{Entity2, Entity2Repo, Entity2Setters, User};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::config::{BehaviorVersion, Region};
use aws_sdk_dynamodb::meta::PKG_VERSION;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client, Error};
use entity_core::*;
use serde::Serialize;

// #[tokio::main]
// async fn main() {
//     main2().await
// }

fn main() {
    println!("{:?}", User::get_schema());
}

async fn main2() {
    let region_provider =
        RegionProviderChain::default_provider().or_else(Region::new("ap-southeast-1"));

    let shared_config = aws_config::defaults(BehaviorVersion::v2025_08_07())
        .region(region_provider)
        .load()
        .await;
    let client = Client::new(&shared_config);

    let m = client.list_tables().send().await.unwrap();
    println!("{:?}", m.table_names);

    let repo = Entity2Repo;

    // ── CREATE ─────────────────────────────────────
    let entity = Entity2 {
        pk: "pk_123".into(),
        attribute2_hello: true,
        attribute3: "sk_partB".into(),
        attribute4: true,
    };

    println!("PK: {}", entity.get_partition_key());
    println!("SK: {}", entity.get_sort_key().unwrap());

    repo.create(entity, client).send2().await.unwrap();

    // ── QUERY ──────────────────────────────────────
    let results = repo.query().where_partition_key("pk_123").send();

    println!("Queried result: {:?}", results);

    // ── UPDATE ─────────────────────────────────────
    repo.update()
        .set_attribute2_hello(true)
        .set_attribute4(false)
        .where_partition_key("pk_123")
        .send();
}
