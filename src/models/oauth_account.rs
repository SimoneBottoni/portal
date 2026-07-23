use jiff::Timestamp;
use toasty::Executor;
use uuid::Uuid;

use crate::models::users::User;

#[derive(Debug, Clone, toasty::Model)]
#[unique(provider, provider_user_id)]
pub struct OAuthAccount {
    #[key]
    #[auto]
    id: Uuid,

    #[index]
    pub user_id: Uuid,

    #[belongs_to(key = user_id, references = id)]
    user: toasty::Deferred<User>,

    provider: String,
    provider_user_id: String,

    #[default(jiff::Timestamp::now())]
    created_at: Timestamp,
    #[update(jiff::Timestamp::now())]
    updated_at: Timestamp,
}

#[allow(clippy::use_self)]
impl OAuthAccount {
    pub async fn create_oauth_account(
        db: &mut impl Executor,
        user_id: Uuid,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<Self, toasty::Error> {
        let account = toasty::create!(OAuthAccount {
            user_id,
            provider,
            provider_user_id
        })
        .exec(db)
        .await?;

        Ok(account)
    }

    pub async fn read_user_by_provider_and_provider_user_id(
        db: &mut impl Executor,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<Self, toasty::Error> {
        OAuthAccount::filter_by_provider_and_provider_user_id(provider, provider_user_id)
            .get(db)
            .await
    }
}
