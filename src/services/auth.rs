use thiserror::Error;
use toasty::Executor;
use uuid::Uuid;

use crate::auth::password::{hash_password, verify_password_hash};
use crate::mailer::MailerContext;
use crate::models::user::User;

use crate::{
    auth::{authn_backend::DUMMY_HASH, oauth_providers::NormalizedProfile},
    models::oauth_account::OAuthAccount,
    views::auth::OAuthProviderKind,
};

#[derive(Debug, Error)]
pub enum AuthServiceError {
    #[error("Invalid email or password")]
    InvalidCredentials,

    #[error("Invalid current password")]
    InvalidCurrentPassword,

    #[error("Database error: {0}")]
    Database(#[from] toasty::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub struct AuthService;

impl AuthService {
    pub async fn register_user(
        db: &mut impl Executor,
        mailer: &MailerContext,
        email: &str,
        raw_password: String,
    ) -> Result<(), AuthServiceError> {
        if User::read_user_by_email(db, email).await.is_ok() {
            return Ok(());
        }

        let password_hash = hash_password(raw_password).await?;

        let _ = User::create_user(db, email, &password_hash).await?;

        if let Err(err) = mailer.send_welcome(email, "").await {
            tracing::error!("Failed to send welcome email to {}: {:?}", email, err);
        }

        Ok(())
    }

    pub async fn find_or_create_oauth_user(
        db: &mut impl Executor,
        provider_kind: OAuthProviderKind,
        profile: &NormalizedProfile,
    ) -> Result<User, AuthServiceError> {
        let provider_str = provider_kind.to_string();

        // Check existing OAuthAccount
        if let Ok(oauth_account) = OAuthAccount::read_user_by_provider_and_provider_user_id(
            db,
            &provider_str,
            &profile.provider_user_id,
        )
        .await
        {
            let mut user = User::read_user_by_id(db, &oauth_account.user_id).await?;

            // Synchronize email if it is different
            if user.email != profile.email {
                let email_taken = User::read_user_by_email(db, &profile.email).await.is_ok();

                if !email_taken {
                    match User::update_user_email(db, user.id, &profile.email).await {
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

        // Check if email exists and link the new provider to the existing user
        if let Ok(existing_user) = User::read_user_by_email(db, &profile.email).await {
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

    pub async fn change_password(
        db: &mut impl Executor,
        user_id: Uuid,
        expected_hash: &str,
        current_password: String,
        new_password: String,
    ) -> Result<(), AuthServiceError> {
        let is_valid = verify_password_hash(current_password, expected_hash).await?;

        if !is_valid {
            return Err(AuthServiceError::InvalidCurrentPassword);
        }

        let new_password_hash = hash_password(new_password).await?;

        User::update_user_password(db, user_id, &new_password_hash).await?;

        Ok(())
    }
}
