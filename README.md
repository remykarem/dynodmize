# dynodmize

Yet another attempt of ODM for DynamoDB.

⚠️This is a pet project!

Define:

```rust
#[derive(Entity)]
#[pk(
	name = "pk",
	value_suffix = "follower",
)]
#[sk(name = "sk")]
struct UserFollower {

	#[pk(prefix = "u")]
	user_id: u32,
	
	#[sk(prefix = "u")]
	follower_id: u32,
}
```

Then

```rust
fn main() {
    let user = UserFollower { user_id: 12345, follower_id: 23456 };
    println!("{}", user.to_item());
}
```

should return

```json
{
	"pk": "u#12345#follower",
	"sk": "u#23456"
}
```

when serialised.
