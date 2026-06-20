use crate::models::StaffUser;
use sqlx::PgPool;

pub async fn list_all_staff(db: &PgPool) -> Result<Vec<StaffUser>, sqlx::Error> {
    sqlx::query_as::<_, StaffUser>(
        r#"
        SELECT id, full_name, email, phone_number, date_of_birth, password_hash,
               role, status, last_login_at, created_at, updated_at
        FROM users
        WHERE role = 'staff'
        ORDER BY created_at DESC, id DESC
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn find_staff_by_id(db: &PgPool, user_id: i64) -> Result<Option<StaffUser>, sqlx::Error> {
    sqlx::query_as::<_, StaffUser>(
        r#"
        SELECT id, full_name, email, phone_number, date_of_birth, password_hash,
               role, status, last_login_at, created_at, updated_at
        FROM users
        WHERE id = $1 AND role = 'staff'
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
}

pub async fn email_exists(db: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"#,
    )
    .bind(email)
    .fetch_one(db)
    .await?;

    Ok(row.0)
}

pub async fn email_exists_for_other(
    db: &PgPool,
    email: &str,
    exclude_id: i64,
) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND id != $2)"#,
    )
    .bind(email)
    .bind(exclude_id)
    .fetch_one(db)
    .await?;

    Ok(row.0)
}

pub async fn create_staff(
    db: &PgPool,
    full_name: &str,
    email: &str,
    phone_number: &str,
    date_of_birth: chrono::NaiveDate,
    password_hash: &str,
    status: &str,
) -> Result<StaffUser, sqlx::Error> {
    sqlx::query_as::<_, StaffUser>(
        r#"
        INSERT INTO users (full_name, email, phone_number, date_of_birth, password_hash, role, status)
        VALUES ($1, $2, $3, $4, $5, 'staff', $6)
        RETURNING id, full_name, email, phone_number, date_of_birth, password_hash,
                  role, status, last_login_at, created_at, updated_at
        "#,
    )
    .bind(full_name)
    .bind(email)
    .bind(phone_number)
    .bind(date_of_birth)
    .bind(password_hash)
    .bind(status)
    .fetch_one(db)
    .await
}

pub async fn update_staff(
    db: &PgPool,
    user_id: i64,
    full_name: &str,
    email: &str,
    phone_number: &str,
    date_of_birth: chrono::NaiveDate,
    status: &str,
) -> Result<StaffUser, sqlx::Error> {
    sqlx::query_as::<_, StaffUser>(
        r#"
        UPDATE users
        SET full_name = $1,
            email = $2,
            phone_number = $3,
            date_of_birth = $4,
            status = $5,
            updated_at = NOW()
        WHERE id = $6 AND role = 'staff'
        RETURNING id, full_name, email, phone_number, date_of_birth, password_hash,
                  role, status, last_login_at, created_at, updated_at
        "#,
    )
    .bind(full_name)
    .bind(email)
    .bind(phone_number)
    .bind(date_of_birth)
    .bind(status)
    .bind(user_id)
    .fetch_one(db)
    .await
}

pub async fn update_staff_password(
    db: &PgPool,
    user_id: i64,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE users
        SET password_hash = $1, updated_at = NOW()
        WHERE id = $2 AND role = 'staff'
        "#,
    )
    .bind(password_hash)
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn delete_staff(db: &PgPool, user_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM users
        WHERE id = $1 AND role = 'staff'
        "#,
    )
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}
