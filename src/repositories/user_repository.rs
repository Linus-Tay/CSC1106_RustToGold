use crate::models::{User, user};
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_user_by_email(db: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, customer_id, full_name, email, phone_number, date_of_birth, password_hash, role, status,
               last_login_at, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(db)
    .await
}

pub async fn find_user_by_username(db: &PgPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User> (
        r#"
        SELECT id, customer_id, username, email, password_hash, role, status, last_login_at, created_at, updated_at
        FROM users
        WHERE username = $1
        "#
    )
    .bind(username)
    .fetch_optional(db)
    .await
}

pub async fn find_user_by_id(db: &PgPool, user_id: Uuid) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User> (
        r#"
        SELECT u.id, u.customer_id, c.full_name, c.date_of_birth, c.phone_number, u.username, u.email, u.password_hash, u.role, u.status, u.last_login_at, u.created_at, u.updated_at
        FROM users u
        LEFT JOIN customers c
        ON u.customer_id = c.id
        WHERE u.id = $1
        "#
    )
    .bind(user_id)
    .fetch_one(db)
    .await
}

pub async fn create_user(
    db: &PgPool,
    customer_id: &Uuid,
    username: &str,
    email: &str,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (customer_id, username, email, password_hash, role, status)
        VALUES ($1, $2, $3, $4, 'customer', 'active')
        RETURNING id, customer_id, username, email, password_hash, role, status,
                  last_login_at, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .fetch_one(db)
    .await
}

pub async fn update_last_login(db: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET last_login_at = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn update_profile(
    db: &PgPool,
    user_id: i64,
    full_name: &str,
    phone_number: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET full_name = $1, phone_number = $2, updated_at = NOW()
        WHERE id = $3
        RETURNING id, customer_id, full_name, email, phone_number, date_of_birth, password_hash, role, status,
                  last_login_at, created_at, updated_at
        "#,
    )
    .bind(full_name)
    .bind(phone_number)
    .bind(user_id)
    .fetch_one(db)
    .await
}