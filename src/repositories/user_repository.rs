use crate::models::User;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

const USER_SELECT: &str = r#"
    SELECT id, customer_id, username, full_name, email, phone_number, date_of_birth,
           password_hash, role, status, last_login_at, created_at, updated_at
    FROM users
"#;

pub async fn find_user_by_email(db: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    let query = format!("{} WHERE lower(email) = lower($1)", USER_SELECT);
    sqlx::query_as::<_, User>(&query)
        .bind(email)
        .fetch_optional(db)
        .await
}

pub async fn find_user_by_username(db: &PgPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    let query = format!("{} WHERE lower(username) = lower($1)", USER_SELECT);
    sqlx::query_as::<_, User>(&query)
        .bind(username)
        .fetch_optional(db)
        .await
}

pub async fn find_user_by_login(db: &PgPool, login: &str) -> Result<Option<User>, sqlx::Error> {
    let query = format!("{} WHERE lower(username) = lower($1) OR lower(email) = lower($1)", USER_SELECT);
    sqlx::query_as::<_, User>(&query)
        .bind(login)
        .fetch_optional(db)
        .await
}

pub async fn find_user_by_id(db: &PgPool, user_id: i64) -> Result<Option<User>, sqlx::Error> {
    let query = format!("{} WHERE id = $1", USER_SELECT);
    sqlx::query_as::<_, User>(&query)
        .bind(user_id)
        .fetch_optional(db)
        .await
}

pub async fn create_customer_user(
    db: &PgPool,
    customer_id: Uuid,
    username: &str,
    full_name: &str,
    email: &str,
    phone_number: &str,
    date_of_birth: NaiveDate,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (customer_id, username, full_name, email, phone_number, date_of_birth, password_hash, role, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'customer', 'active')
        RETURNING id, customer_id, username, full_name, email, phone_number, date_of_birth,
                  password_hash, role, status, last_login_at, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .bind(username)
    .bind(full_name)
    .bind(email)
    .bind(phone_number)
    .bind(date_of_birth)
    .bind(password_hash)
    .fetch_one(db)
    .await
}

// Backwards-compatible name for older imports.
pub async fn create_customer(
    db: &PgPool,
    customer_id: Uuid,
    full_name: &str,
    email: &str,
    phone_number: &str,
    date_of_birth: NaiveDate,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    let fallback_username = email.split('@').next().unwrap_or(email).to_lowercase();
    create_customer_user(
        db,
        customer_id,
        &fallback_username,
        full_name,
        email,
        phone_number,
        date_of_birth,
        password_hash,
    )
    .await
}

pub async fn update_last_login(db: &PgPool, user_id: i64) -> Result<(), sqlx::Error> {
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
        RETURNING id, customer_id, username, full_name, email, phone_number, date_of_birth,
                  password_hash, role, status, last_login_at, created_at, updated_at
        "#,
    )
    .bind(full_name)
    .bind(phone_number)
    .bind(user_id)
    .fetch_one(db)
    .await
}
