use jiff::Timestamp;
use uuid::Uuid;

#[derive(Debug, Clone, toasty::Model)]
pub struct User {
    #[key]
    #[auto]
    pub id: Uuid,

    #[unique]
    email: String,
    pub password_hash: String,

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

    pub async fn read_user_by_email(
        db: &mut toasty::Db,
        email: String,
    ) -> Result<Self, toasty::Error> {
        User::filter_by_email(email).get(db).await
    }

    pub async fn read_user_by_id(db: &mut toasty::Db, id: &Uuid) -> Result<Self, toasty::Error> {
        User::filter_by_id(id).get(db).await
    }

    pub async fn update_user_password(
        db: &mut toasty::Db,
        id: Uuid,
        password_hash: &str,
    ) -> anyhow::Result<()> {
        User::update_by_id(id)
            .password_hash(password_hash)
            .exec(db)
            .await?;

        Ok(())
    }
}
