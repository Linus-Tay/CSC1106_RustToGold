use crate::models::User;
use chrono::NaiveDate;
use sqlx::PgPool;

pub async fn find_user_by_email(db: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, full_name, email, phone_number, date_of_birth, password_hash, role, status,
               last_login_at, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(db)
    .await
}

pub async fn find_user_by_id(db: &PgPool, user_id: i64) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, full_name, email, phone_number, date_of_birth, password_hash, role, status,
               last_login_at, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
}

pub async fn create_customer(
    db: &PgPool,
    full_name: &str,
    email: &str,
    phone_number: &str,
    date_of_birth: NaiveDate,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (full_name, email, phone_number, date_of_birth, password_hash, role, status)
        VALUES ($1, $2, $3, $4, $5, 'customer', 'active')
        RETURNING id, full_name, email, phone_number, date_of_birth, password_hash, role, status,
                  last_login_at, created_at, updated_at
        "#,
    )
    .bind(full_name)
    .bind(email)
    .bind(phone_number)
    .bind(date_of_birth)
    .bind(password_hash)
    .fetch_one(db)
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
        RETURNING id, full_name, email, phone_number, date_of_birth, password_hash, role, status,
                  last_login_at, created_at, updated_at
        "#,
    )
    .bind(full_name)
    .bind(phone_number)
    .bind(user_id)
    .fetch_one(db)
    .await
}
