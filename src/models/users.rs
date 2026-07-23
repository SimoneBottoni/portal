use jiff::Timestamp;
use toasty::Executor;
use uuid::Uuid;

use crate::{
    auth::{authn_backend::DUMMY_HASH, oauth_providers::NormalizedProfile},
    models::oauth_account::OAuthAccount,
    views::auth::OAuthProviderKind,
};

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

    pub async fn find_or_create_oauth_user(
        db: &mut impl Executor,
        provider_kind: OAuthProviderKind,
        profile: &NormalizedProfile,
    ) -> Result<Self, toasty::Error> {
        let provider_str = provider_kind.to_string();

        // Check OAuthAccount
        if let Ok(oauth_account) = OAuthAccount::read_user_by_provider_and_provider_user_id(
            db,
            &provider_str,
            &profile.provider_user_id,
        )
        .await
        {
            let mut user = Self::read_user_by_id(db, &oauth_account.user_id).await?;

            // Synchronize email if it is different
            if user.email != profile.email {
                let email_taken = Self::read_user_by_email(db, &profile.email).await.is_ok();

                if !email_taken {
                    match Self::update_user_email(db, user.id, &profile.email).await {
                        Ok(_) => {
                            user.email = profile.email.clone();
                        }
                        Err(err) => {
                            tracing::warn!(
                                user_id = %user.id,
                                error = %err,
                                "Failed to synchronize updated OAuth email"
                            );
                        }
                    }
                }
            }

            return Ok(user);
        }

        // Check email and link the new provider to the existing user
        if let Ok(existing_user) = Self::read_user_by_email(db, &profile.email).await {
            OAuthAccount::create_oauth_account(
                db,
                existing_user.id,
                &provider_str,
                &profile.provider_user_id,
            )
            .await?;

            return Ok(existing_user);
        }

        // User not found, creating
        let mut tx = db.transaction().await?;

        let user = User::create_user(&mut tx, &profile.email, DUMMY_HASH).await?;

        OAuthAccount::create_oauth_account(
            &mut tx,
            user.id,
            &provider_str,
            &profile.provider_user_id,
        )
        .await?;

        tx.commit().await?;

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
