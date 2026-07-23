use jiff::Timestamp;
use toasty::Executor;
use uuid::Uuid;

use crate::models::oauth_account::OAuthAccount;

#[derive(Debug, Clone, toasty::Model)]
pub struct User {
    #[key]
    #[auto]
    pub id: Uuid,

    #[unique]
    pub email: String,
    pub password_hash: String,

    #[default(false)]
    is_admin: bool,

    email_verified_at: Option<Timestamp>,

    #[has_many]
    oauth_accounts: toasty::Deferred<Vec<OAuthAccount>>,

    #[default(jiff::Timestamp::now())]
    created_at: Timestamp,
    #[update(jiff::Timestamp::now())]
    updated_at: Timestamp,
}

#[allow(clippy::use_self)]
impl User {
    pub async fn create_user(
        db: &mut impl Executor,
        email: &str,
        password_hash: &str,
    ) -> Result<Self, toasty::Error> {
        let user = toasty::create!(User {
            email,
            password_hash
        })
        .exec(db)
        .await?;

        Ok(user)
    }

    pub async fn read_user_by_email(
        db: &mut impl Executor,
        email: &str,
    ) -> Result<Self, toasty::Error> {
        User::filter_by_email(email).get(db).await
    }

    pub async fn read_user_by_id(db: &mut impl Executor, id: &Uuid) -> Result<Self, toasty::Error> {
        User::filter_by_id(id).get(db).await
    }

    pub async fn update_user_email(
        db: &mut impl Executor,
        id: Uuid,
        email: &str,
    ) -> Result<(), toasty::Error> {
        User::update_by_id(id).email(email).exec(db).await
    }

    pub async fn update_user_password(
        db: &mut impl Executor,
        id: Uuid,
        password_hash: &str,
    ) -> Result<(), toasty::Error> {
        User::update_by_id(id)
            .password_hash(password_hash)
            .exec(db)
            .await
    }
}
