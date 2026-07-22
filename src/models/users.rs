use jiff::Timestamp;
use uuid::Uuid;

#[derive(Debug, toasty::Model)]
pub struct User {
    #[key]
    #[auto]
    id: Uuid,

    #[unique]
    email: String,
    password_hash: String,

    #[default(false)]
    is_admin: bool,

    email_verified_at: Option<Timestamp>,

    #[default(jiff::Timestamp::now())]
    created_at: Timestamp,
    #[update(jiff::Timestamp::now())]
    updated_at: Timestamp,
}

#[allow(clippy::use_self)]
impl User {
    pub async fn create_user(
        db: &mut toasty::Db,
        email: &str,
        password_hash: &str,
    ) -> anyhow::Result<()> {
        toasty::create!(User {
            email,
            password_hash
        })
        .exec(db)
        .await?;

        Ok(())
    }
}
