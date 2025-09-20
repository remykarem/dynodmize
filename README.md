# dynodmize

Yet another attempt of ODM for DynamoDB.

Defining this

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
	follower_id: usize,
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
