use password_auth::{generate_hash, verify_password};
use tokio::task;

pub const DUMMY_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$cGxlYXNlZG9ub3R1c2U$placeholderhashforconstanttimecheck";

pub async fn hash_password(password: String) -> anyhow::Result<String> {
    let hash = task::spawn_blocking(move || generate_hash(password)).await?;
    Ok(hash)
}

pub async fn verify_password_hash(password: String, hash: &str) -> anyhow::Result<bool> {
    let hash = hash.to_string();
    let is_valid = task::spawn_blocking(move || verify_password(password, &hash).is_ok()).await?;
    Ok(is_valid)
}
