use crate::models::{PersonalLoan, Product, Transaction};
use sqlx::{PgPool, Postgres, Transaction as DbTransaction};
use uuid::Uuid;

pub async fn find_primary_active_product(db: &PgPool, customer_id: Uuid) -> Result<Product, sqlx::Error> {
    sqlx::query_as::<_, Product>(
        r#"
        SELECT id, customer_id, account_number, product_id, product_type, balance_cents, status, created_at, updated_at
        FROM customer_products
        WHERE customer_id = $1 AND status = 'active'
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .bind(customer_id)
    .fetch_one(db)
    .await
}

pub async fn list_personal_loans_by_customer(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<Vec<PersonalLoan>, sqlx::Error> {
    sqlx::query_as::<_, PersonalLoan>(
        r#"
        SELECT id, customer_id, funding_product_id, purpose, principal_cents, annual_rate_bps,
               term_months, monthly_payment_cents, outstanding_cents, status, created_at, updated_at
        FROM personal_loans
        WHERE customer_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
}

pub async fn create_personal_loan(
    db: &PgPool,
    customer_id: Uuid,
    product_id: Uuid,
    purpose: &str,
    amount_cents: i64,
    annual_rate_bps: i32,
    term_months: i32,
    monthly_payment_cents: i64,
) -> Result<PersonalLoan, sqlx::Error> {
    let product = sqlx::query_as::<_, Product>(
        r#"
        SELECT id, customer_id, account_number, product_id, product_type, balance_cents, status, created_at, updated_at
        FROM customer_products
        WHERE id = $1 AND customer_id = $2 AND status = 'active'
        "#,
    )
    .bind(product_id)
    .bind(customer_id)
    .fetch_one(db)
    .await?;

    sqlx::query_as::<_, PersonalLoan>(
        r#"
        INSERT INTO personal_loans (
            customer_id, funding_product_id, purpose, principal_cents, annual_rate_bps,
            term_months, monthly_payment_cents, outstanding_cents, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, 0, 'pending')
        RETURNING id, customer_id, funding_product_id, purpose, principal_cents, annual_rate_bps,
                  term_months, monthly_payment_cents, outstanding_cents, status, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .bind(product.id)
    .bind(purpose)
    .bind(amount_cents)
    .bind(annual_rate_bps)
    .bind(term_months)
    .bind(monthly_payment_cents)
    .fetch_one(db)
    .await
}

pub async fn pay_personal_loan(
    db: &PgPool,
    customer_id: Uuid,
    loan_id: Uuid,
    amount_cents: i64,
) -> Result<PersonalLoan, sqlx::Error> {
    let mut tx = db.begin().await?;

    let loan = sqlx::query_as::<_, PersonalLoan>(
        r#"
        SELECT id, customer_id, funding_product_id, purpose, principal_cents, annual_rate_bps,
               term_months, monthly_payment_cents, outstanding_cents, status, created_at, updated_at
        FROM personal_loans
        WHERE id = $1 AND customer_id = $2 AND status = 'active' AND outstanding_cents > 0
        FOR UPDATE
        "#,
    )
    .bind(loan_id)
    .bind(customer_id)
    .fetch_one(&mut *tx)
    .await?;

    let product = lock_product_by_id(&mut tx, customer_id, loan.funding_product_id).await?;
    let payment_cents = amount_cents.min(loan.outstanding_cents);
    let new_product_balance = product.balance_cents - payment_cents;
    let new_outstanding = loan.outstanding_cents - payment_cents;
    let new_status = if new_outstanding == 0 { "fully_paid" } else { "active" };

    sqlx::query(
        r#"
        UPDATE customer_products
        SET balance_cents = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(new_product_balance)
    .bind(product.id)
    .execute(&mut *tx)
    .await?;

    let updated = sqlx::query_as::<_, PersonalLoan>(
        r#"
        UPDATE personal_loans
        SET outstanding_cents = $1, status = $2, updated_at = NOW()
        WHERE id = $3
        RETURNING id, customer_id, funding_product_id, purpose, principal_cents, annual_rate_bps,
                  term_months, monthly_payment_cents, outstanding_cents, status, created_at, updated_at
        "#,
    )
    .bind(new_outstanding)
    .bind(new_status)
    .bind(loan.id)
    .fetch_one(&mut *tx)
    .await?;

    insert_product_transaction(
        &mut tx,
        product.id,
        customer_id,
        "loan_payment",
        payment_cents,
        new_product_balance,
        Some("Personal loan repayment"),
    )
    .await?;

    tx.commit().await?;
    Ok(updated)
}

async fn lock_product_by_id(
    tx: &mut DbTransaction<'_, Postgres>,
    customer_id: Uuid,
    product_id: Uuid,
) -> Result<Product, sqlx::Error> {
    sqlx::query_as::<_, Product>(
        r#"
        SELECT id, customer_id, account_number, product_id, product_type, balance_cents, status, created_at, updated_at
        FROM customer_products
        WHERE id = $1 AND customer_id = $2 AND status = 'active'
        FOR UPDATE
        "#,
    )
    .bind(product_id)
    .bind(customer_id)
    .fetch_one(&mut **tx)
    .await
}

async fn insert_product_transaction(
    tx: &mut DbTransaction<'_, Postgres>,
    product_id: Uuid,
    customer_id: Uuid,
    transaction_type: &str,
    amount_cents: i64,
    balance_after_cents: i64,
    description: Option<&str>,
) -> Result<Transaction, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (product_id, customer_id, transaction_type, amount_cents, balance_after_cents, description)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, product_id, customer_id, transaction_type, amount_cents, balance_after_cents, description, created_at
        "#,
    )
    .bind(product_id)
    .bind(customer_id)
    .bind(transaction_type)
    .bind(amount_cents)
    .bind(balance_after_cents)
    .bind(description)
    .fetch_one(&mut **tx)
    .await
}
