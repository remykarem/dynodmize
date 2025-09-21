mod entity2tings;
mod recurring_payments;
mod social_network;

use crate::entity2tings::{ComplaintComments, Entity2Repo, MyEntity2, MyEntity2Setters};
use crate::recurring_payments::AccountReceiptSubscription;
use crate::social_network::UserCount;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::config::{BehaviorVersion, Region};
use entity_core::Entity2;
use entity_core::*;
use entity_macros::Dynodmize;
// #[tokio::main]
// async fn main() {
//     main2().await
// }

#[derive(Dynodmize)]
#[pk(name = "pk")]
#[sk(name = "sk")]
struct Timeline {
    #[pk(prefix = "u")]
    user_id: u32,

    #[sk(prefix = "p", order = 0)]
    post_id: u32,

    #[sk(prefix = "u", order = 1)]
    following_id: u32,
}

#[derive(Dynodmize)]
struct UserItem {
    #[pk(prefix = "u")]
    username: String,

    #[sk(prefix = "ITEM")]
    item_id: u32,
}

fn main() {
    let ent = ComplaintComments {
        complaint_id: 123,
        comment_date: "somedate".to_string(),
        comment_dates: "somedatess".to_string(),
        comment_id: 456,
        attribute2: "d".to_string(),
    };
    println!("{}", serde_json::to_string_pretty(&ent.to_item()).unwrap());

    let user_count = UserCount {
        user_id: 123,
        followers: 3,
        followings: 1000,
        posts: 4,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&user_count.to_item()).unwrap()
    );

    let user_item = UserItem {
        username: "user001".to_string(),
        item_id: 999,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&user_item.to_item()).unwrap()
    );

    let account_receipt_subscription = AccountReceiptSubscription {
        account_id: 123,
        last_reminder_date: "2025-09-20".to_string(),
        next_reminder_date: "2025-09-27".to_string(),
        subscription_id: 987,
        sku: 999,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&account_receipt_subscription.to_item()).unwrap()
    );

    let timeline = Timeline {
        user_id: 987,
        post_id: 111,
        following_id: 2344224,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&timeline.to_item()).unwrap()
    );
}

#[allow(dead_code)]
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
