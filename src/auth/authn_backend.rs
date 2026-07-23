use axum_login::AuthnBackend;

use crate::{
    auth::password::verify_password_hash, models::users::User, views::auth::UserCredentials,
};

const DUMMY_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$cGxlYXNlZG9ub3R1c2U$placeholderhashforconstanttimecheck";

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Authentication failed.")]
    AuthFailed,

    #[error(transparent)]
    Database(#[from] toasty::Error),
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: toasty::Db,
}

impl Backend {
    pub const fn new(db: toasty::Db) -> Self {
        Self { db }
    }
}

impl AuthnBackend for Backend {
    type User = User;

    type Credentials = UserCredentials;

    type Error = AuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let mut db: toasty::Db = self.db.clone();

        let (user, expected_hash) = match User::read_user_by_email(&mut db, creds.email).await {
            Ok(user) => {
                let hash = user.password_hash.clone();
                (Some(user), hash)
            }
            Err(err) if err.is_record_not_found() => (None, DUMMY_HASH.to_string()),
            Err(err) => return Err(AuthError::Database(err)),
        };

        let password_hash = creds.password;

        let is_valid = verify_password_hash(password_hash, &expected_hash)
            .await
            .map_err(|_| AuthError::AuthFailed)?;

        Ok(user.filter(|_| is_valid))
    }

    async fn get_user(
        &self,
        user_id: &axum_login::UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        let mut db: toasty::Db = self.db.clone();

        match User::read_user_by_id(&mut db, user_id).await {
            Ok(user) => Ok(Some(user)),
            Err(err) if err.is_record_not_found() => Ok(None),
            Err(_) => Err(AuthError::AuthFailed),
        }
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
