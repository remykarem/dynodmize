use entity_macros::Entity;

#[derive(Entity)]
#[pk(name = "pk")]
#[sk(
    name = "sk",
    value = "count",
)]
struct UserCount {

    #[pk(prefix = "u")]
    user_id: u32,
    followers: usize,
    followings: usize,
    posts: usize,

}
