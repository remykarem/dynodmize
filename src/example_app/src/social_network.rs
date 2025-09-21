use entity_macros::Dynodmize;

#[derive(Dynodmize)]
#[pk(name = "pk")]
#[sk(name = "sk", value = "count")]
pub struct UserCount {
    #[pk(prefix = "u")]
    pub(crate) user_id: u32,
    #[nk]
    pub(crate) followers: usize,
    #[nk]
    pub(crate) followings: usize,
    #[nk]
    pub(crate) posts: usize,
}
