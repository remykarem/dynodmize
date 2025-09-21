# dynodmize

Yet another attempt of ODM for DynamoDB.

⚠️This is a pet project!

Declarative way to define the

* primary key (`pk`)
* sort key (`sk`)
* non-key (`nk`)

over a struct.

Aims to make single-table design less painful 😭

## Examples

Examples based
on [Data modeling schema design packages in DynamoDB](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/data-modeling-schemas.html).

### Example 1

Define

```rust
#[derive(Dynodmize)]
struct UserItem {

	#[pk]
	username: String,
	
	#[sk]
	item_id: u32
}
```

Then

```rust
fn main() {
    let user_item = UserItem {
        username: "user001".to_string(),
        item_id: 999,
    };
    println!("{}", user_item.to_item());
}
```

will give you

```json
{
  "item_id": "999",
  "username": "user001"
}
```

### Example 2

```rust
#[derive(Dynodmize)]
struct UserItem {

	#[pk(prefix = "u")]
	username: String,
	
	#[sk(prefix = "ITEM")]
	item_id: u32
}
```

```json
{
  "item_id": "ITEM#999",
  "username": "u#user001"
}
```

### Example 3


```rust
#[derive(Dynodmize)]
#[pk(
    name = "pk",
    suffix = "follower",
)]
#[sk(name = "sk")]
struct UserFollower {
    #[pk(prefix = "u")]
    user_id: u32,

    #[sk(prefix = "u")]
    follower_id: u32,
}
```

```json
{
  "pk": "u#12345#follower",
  "sk": "u#23456"
}
```

### Example 4

Composite keys

```rust
#[derive(Dynodmize)]
#[pk(
	name = "pk",
	suffix = "timeline",
)]
#[sk(name = "sk")]
struct Timeline {

	#[pk(prefix = "u")]
	user_id: u32,
	
	#[sk(prefix = "p", order = 0)]
	post_id: u32,
	
	#[sk(prefix = "u", order = 1)]
	following_id: u32,

}
```

```json
{
  "pk": "u#987#timeline",
  "sk": "p#111#u#2344224"
}
```

### Example 5

```rust
#[derive(Dynodmize)]
#[pk(name = "NextReminderDate")]
#[sk(name = "LastReminderDate")]
#[nk(name = "SK")]
#[nk(name = "PK")]
pub struct AccountReceiptSubscription {
    #[pk]
    pub next_reminder_date: String,

    #[sk]
    pub last_reminder_date: String,

    #[nk(name = "SK", prefix = "SUB", order = 1)]
    pub subscription_id: u32,

    #[nk(name = "SK", prefix = "SKU", order = 0)]
    #[nk]
    pub sku: u32,

    #[nk(name = "PK", prefix = "ACC")]
    pub account_id: u32,
}
```

```json
{
  "NextReminderDate": "2025-09-27",
  "LastReminderDate": "2025-09-20",
  "PK": "ACC#123",
  "SK": "SUB#987#SKU#999",
  "sku": "999"
}
```