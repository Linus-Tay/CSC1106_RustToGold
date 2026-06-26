use crate::models::{BankAccount, Transaction};
use sqlx::{PgPool, Postgres, Transaction as DbTransaction};
use uuid::Uuid;

pub async fn create_primary_account(
    db: &PgPool,
    user_id: i64,
    account_type: &str,
) -> Result<BankAccount, sqlx::Error> {
    let account_number = format!("RTG-{}", Uuid::new_v4().simple());

    sqlx::query_as::<_, BankAccount>(
        r#"
        INSERT INTO bank_accounts (user_id, account_number, account_type, balance_cents, status)
        VALUES ($1, $2, $3, 0, 'active')
        RETURNING id, user_id, account_number, account_type, balance_cents, status, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(account_number)
    .bind(account_type)
    .fetch_one(db)
    .await
}

pub async fn find_primary_account_by_user_id(
    db: &PgPool,
    user_id: i64,
) -> Result<Option<BankAccount>, sqlx::Error> {
    sqlx::query_as::<_, BankAccount>(
        r#"
        SELECT id, user_id, account_number, account_type, balance_cents, status, created_at, updated_at
        FROM bank_accounts
        WHERE user_id = $1
        ORDER BY id ASC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
}

pub async fn deposit_to_primary_account(
    db: &PgPool,
    user_id: i64,
    amount_cents: i64,
    description: Option<&str>,
) -> Result<(BankAccount, Transaction), sqlx::Error> {
    let mut tx = db.begin().await?;
    let account = lock_primary_account(&mut tx, user_id).await?;
    let new_balance = account.balance_cents + amount_cents;

    let updated_account = sqlx::query_as::<_, BankAccount>(
        r#"
        UPDATE bank_accounts
        SET balance_cents = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, user_id, account_number, account_type, balance_cents, status, created_at, updated_at
        "#,
    )
    .bind(new_balance)
    .bind(account.id)
    .fetch_one(&mut *tx)
    .await?;

    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (account_id, user_id, transaction_type, amount_cents, balance_after_cents, description)
        VALUES ($1, $2, 'deposit', $3, $4, $5)
        RETURNING id, account_id, user_id, transaction_type, amount_cents, balance_after_cents, description, created_at
        "#,
    )
    .bind(account.id)
    .bind(user_id)
    .bind(amount_cents)
    .bind(new_balance)
    .bind(description)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((updated_account, transaction))
}

async fn lock_primary_account(
    tx: &mut DbTransaction<'_, Postgres>,
    user_id: i64,
) -> Result<BankAccount, sqlx::Error> {
    sqlx::query_as::<_, BankAccount>(
        r#"
        SELECT id, user_id, account_number, account_type, balance_cents, status, created_at, updated_at
        FROM bank_accounts
        WHERE user_id = $1 AND status = 'active'
        ORDER BY id ASC
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
}
