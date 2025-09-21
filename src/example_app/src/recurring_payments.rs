use entity_macros::Dynodmize;

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
