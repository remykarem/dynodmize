mod entity2tings;

use crate::entity2tings::{Entity2Repo, MyEntity2, MyEntity2Setters, User};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::config::{BehaviorVersion, Region};
use aws_sdk_dynamodb::Client;
use entity_core::*;
use serde::Serialize;
use entity_core::Entity2;
// #[tokio::main]
// async fn main() {
//     main2().await
// }

fn main() {
    println!("{:?}", User::get_schema());
    let user = User { 
        last_name: "D".to_string(),

        first_name: "dd".to_string(),
        attribute2: "ss".to_string(),
        attribute3: "dgg".to_string(),
        attribute4: "fhjj".to_string(),
        attribute5: "uu".to_string(),
    };
    println!("{}", user.to_item());
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
    let entity = MyEntity2 {
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
