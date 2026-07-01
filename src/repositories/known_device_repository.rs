// Repository layer: isolates SQLx queries so services do not depend on raw database code.

use crate::models::{KnownDevice, User};
use sqlx::PgPool;
use uuid::Uuid;

const USER_SELECT: &str = r#"
    SELECT id, customer_id, username, email, password_hash, role, status, last_login_at, created_at, updated_at
    FROM users
"#;

// Reads find user by email data from the database.
pub async fn find_device_by_hashed_token(db: &PgPool, hashed_token: &str) -> Result<Option<KnownDevice>, sqlx::Error> {;
    sqlx::query_as::<_, KnownDevice>(r#"
        SELECT id, hash, token_hash, user_id, last_used
        FROM known_device_repository
        WHERE token_hash = $1

    "#)
    .bind(hashed_token)
    .fetch_optional(db)
    .await
}

// Reads find user by username data from the database.
pub async fn find_user_by_username(db: &PgPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    let query = format!("{} WHERE lower(username) = lower($1)", USER_SELECT);
    sqlx::query_as::<_, User>(&query)
        .bind(username)
        .fetch_optional(db)
        .await
}

// Reads find user by login data from the database.
pub async fn find_user_by_login(db: &PgPool, login: &str) -> Result<Option<User>, sqlx::Error> {
    let query = format!("{} WHERE lower(username) = lower($1) OR lower(email) = lower($1)", USER_SELECT);
    sqlx::query_as::<_, User>(&query)
        .bind(login)
        .fetch_optional(db)
        .await
}

// Reads find user by id data from the database.
pub async fn find_user_by_id(db: &PgPool, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
    let query = format!("{} WHERE id = $1", USER_SELECT);
    sqlx::query_as::<_, User>(&query)
        .bind(user_id)
        .fetch_optional(db)
        .await
}

// Persists the create customer user database change.
pub async fn create_customer_user(
    db: &PgPool,
    customer_id: Uuid,
    username: &str,
    email: &str,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (customer_id, username, email, password_hash, role, status)
        VALUES ($1, $2, $3, $4, 'customer', 'active')
        RETURNING id, customer_id, username, email, password_hash, role, status, last_login_at, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .fetch_one(db)
    .await
}

// Backwards-compatible name for older imports. Customer identity belongs to customers, not users.
pub async fn create_known_device(
    db: &PgPool,
    user_id: &Uuid,
    hashed_token: &str,
) -> Result<KnownDevice, sqlx::Error> {
     sqlx::query_as::<_, KnownDevice>(
        r#"
        INSERT INTO known_devices (user_id, token_hash)
        VALUES ($1, $2)
        RETURNING id, user_id, token_hash, last_used
        "#,
    )
    .bind(user_id)
    .bind(hashed_token)
    .fetch_one(db)
    .await
}

// Persists the update last login database change.
pub async fn update_last_login(db: &PgPool, hashed_token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE known_device SET last_used = NOW() WHERE token_hash = $1")
        .bind(hashed_token)
        .execute(db)
        .await?;

    Ok(())
}
