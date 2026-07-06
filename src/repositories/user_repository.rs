
use crate::models::User;
use sqlx::PgPool;
use uuid::Uuid;

const USER_SELECT: &str = r#"
    SELECT id, customer_id, username, email, password_hash, role, status, last_login_at, created_at, updated_at
    FROM users
"#;

// Query find user by login
pub async fn find_user_by_login(db: &PgPool, login: &str) -> Result<Option<User>, sqlx::Error> {
    let query = format!("{} WHERE lower(username) = lower($1) OR lower(email) = lower($1)", USER_SELECT);
    sqlx::query_as::<_, User>(&query)
        .bind(login)
        .fetch_optional(db)
        .await
}

// Query find user by id
pub async fn find_user_by_id(db: &PgPool, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
    let query = format!("{} WHERE id = $1", USER_SELECT);
    sqlx::query_as::<_, User>(&query)
        .bind(user_id)
        .fetch_optional(db)
        .await
}

// Persist create customer user
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

// Persist update last login
pub async fn update_last_login(db: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET last_login_at = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(db)
        .await?;

    Ok(())
}

// Persist update password
pub async fn update_password(
    db: &PgPool,
    user_id: Uuid,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET password_hash = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, customer_id, username, email, password_hash, role, status, last_login_at, created_at, updated_at
        "#,
    )
    .bind(password_hash)
    .bind(user_id)
    .fetch_one(db)
    .await
}
