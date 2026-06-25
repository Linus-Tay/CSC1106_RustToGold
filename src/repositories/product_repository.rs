use crate::models::{BankAccount, Product, Transaction, product};
use sqlx::{PgPool, Postgres, Transaction as DbTransaction};
use uuid::Uuid;

pub async fn get_product_by_user_id(db: &PgPool, customer_id: &Uuid, product_id: &String) -> Result<Option<Product>, sqlx::Error> {
    sqlx::query_as::<_, Product> (
        r#"
        SELECT * FROM customer_products WHERE product_id = $1 AND customer_id = $2
        "#
    )
    .bind(product_id)
    .bind(customer_id)
    .fetch_optional(db)
    .await
}

pub async fn get_product_by_account_number(db: &PgPool, account_number: &String) -> Result<Option<Product>, sqlx::Error> {
    sqlx::query_as::<_, Product> (
        r#"
        SELECT * FROM customer_products WHERE account_number = $1
        "#
    )
    .bind(account_number)
    .fetch_optional(db)
    .await
}

pub async fn insert_product(db: &PgPool, customer_id: Uuid, product_id: String, account_number: String) -> Result<Product, sqlx::Error> {
    sqlx::query_as::<_, Product>(
        r#"
        INSERT INTO customer_products (customer_id, product_id, account_number, balance_cents, status)
        VALUES ($1, $2, $3, 0, $4)
        RETURNING id, customer_id, account_number, product_id, balance_cents, status, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .bind(product_id)
    .bind(account_number)
    .bind(product::AccountStatus::PENDING)
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

// pub async fn deposit_into_product(
//     db: &PgPool,
//     customer_id: &Uuid,
//     account_number: &String,
//     amount_cents: i64,
//     description: Option<&str>,
// ) -> Result<(Product, Transaction), sqlx::Error> {
//     let mut tx = db.begin().await?;
//     let account = lock_primary_account(&mut tx, user_id).await?;
//     let new_balance = account.balance_cents + amount_cents;

//     let updated_account = sqlx::query_as::<_, BankAccount>(
//         r#"
//         UPDATE bank_accounts
//         SET balance_cents = $1, updated_at = NOW()
//         WHERE id = $2
//         RETURNING id, user_id, account_number, account_type, balance_cents, status, created_at, updated_at
//         "#,
//     )
//     .bind(new_balance)
//     .bind(account.id)
//     .fetch_one(&mut *tx)
//     .await?;

//     let transaction = sqlx::query_as::<_, Transaction>(
//         r#"
//         INSERT INTO transactions (account_id, user_id, transaction_type, amount_cents, balance_after_cents, description)
//         VALUES ($1, $2, 'deposit', $3, $4, $5)
//         RETURNING id, account_id, user_id, transaction_type, amount_cents, balance_after_cents, description, created_at
//         "#,
//     )
//     .bind(account.id)
//     .bind(user_id)
//     .bind(amount_cents)
//     .bind(new_balance)
//     .bind(description)
//     .fetch_one(&mut *tx)
//     .await?;

//     tx.commit().await?;
//     Ok((updated_account, transaction))
// }

// async fn lock_primary_account(
//     tx: &mut DbTransaction<'_, Postgres>,
//     user_id: i64,
// ) -> Result<BankAccount, sqlx::Error> {
//     sqlx::query_as::<_, BankAccount>(
//         r#"
//         SELECT id, user_id, account_number, account_type, balance_cents, status, created_at, updated_at
//         FROM bank_accounts
//         WHERE user_id = $1 AND status = 'active'
//         ORDER BY id ASC
//         LIMIT 1
//         FOR UPDATE
//         "#,
//     )
//     .bind(user_id)
//     .fetch_one(&mut **tx)
//     .await
// }
