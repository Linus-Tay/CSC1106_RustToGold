use crate::models::{HomeLoanApplication, Product, Transaction};
use sqlx::{PgPool, Postgres, Transaction as DbTransaction};
use uuid::Uuid;

pub async fn list_home_loans_by_customer(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<Vec<HomeLoanApplication>, sqlx::Error> {
    sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        SELECT id, customer_id, account_product_id, property_type, property_value_cents,
               down_payment_cents, loan_amount_cents, annual_rate_bps, term_years,
               monthly_payment_cents, outstanding_cents, status, reviewed_by, reviewed_at,
               created_at, updated_at
        FROM home_loan_applications
        WHERE customer_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
}

pub async fn list_all_home_loans(db: &PgPool) -> Result<Vec<HomeLoanApplication>, sqlx::Error> {
    sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        SELECT id, customer_id, account_product_id, property_type, property_value_cents,
               down_payment_cents, loan_amount_cents, annual_rate_bps, term_years,
               monthly_payment_cents, outstanding_cents, status, reviewed_by, reviewed_at,
               created_at, updated_at
        FROM home_loan_applications
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn create_home_loan_application(
    db: &PgPool,
    customer_id: Uuid,
    account_product_id: Option<Uuid>,
    property_type: &str,
    property_value_cents: i64,
    down_payment_cents: i64,
    loan_amount_cents: i64,
    annual_rate_bps: i32,
    term_years: i32,
    monthly_payment_cents: i64,
) -> Result<HomeLoanApplication, sqlx::Error> {
    sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        INSERT INTO home_loan_applications (
            customer_id, account_product_id, property_type, property_value_cents,
            down_payment_cents, loan_amount_cents, annual_rate_bps, term_years,
            monthly_payment_cents, outstanding_cents, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0, 'pending')
        RETURNING id, customer_id, account_product_id, property_type, property_value_cents,
                  down_payment_cents, loan_amount_cents, annual_rate_bps, term_years,
                  monthly_payment_cents, outstanding_cents, status, reviewed_by, reviewed_at,
                  created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .bind(account_product_id)
    .bind(property_type)
    .bind(property_value_cents)
    .bind(down_payment_cents)
    .bind(loan_amount_cents)
    .bind(annual_rate_bps)
    .bind(term_years)
    .bind(monthly_payment_cents)
    .fetch_one(db)
    .await
}

pub async fn approve_home_loan(
    db: &PgPool,
    staff_user_id: Uuid,
    application_id: Uuid,
) -> Result<HomeLoanApplication, sqlx::Error> {
    sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        UPDATE home_loan_applications
        SET status = 'approved',
            outstanding_cents = loan_amount_cents,
            reviewed_by = $1,
            reviewed_at = NOW(),
            updated_at = NOW()
        WHERE id = $2 AND status = 'pending'
        RETURNING id, customer_id, account_product_id, property_type, property_value_cents,
                  down_payment_cents, loan_amount_cents, annual_rate_bps, term_years,
                  monthly_payment_cents, outstanding_cents, status, reviewed_by, reviewed_at,
                  created_at, updated_at
        "#,
    )
    .bind(staff_user_id)
    .bind(application_id)
    .fetch_one(db)
    .await
}

pub async fn reject_home_loan(
    db: &PgPool,
    staff_user_id: Uuid,
    application_id: Uuid,
) -> Result<HomeLoanApplication, sqlx::Error> {
    sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        UPDATE home_loan_applications
        SET status = 'rejected', reviewed_by = $1, reviewed_at = NOW(), updated_at = NOW()
        WHERE id = $2 AND status = 'pending'
        RETURNING id, customer_id, account_product_id, property_type, property_value_cents,
                  down_payment_cents, loan_amount_cents, annual_rate_bps, term_years,
                  monthly_payment_cents, outstanding_cents, status, reviewed_by, reviewed_at,
                  created_at, updated_at
        "#,
    )
    .bind(staff_user_id)
    .bind(application_id)
    .fetch_one(db)
    .await
}

pub async fn pay_home_loan(
    db: &PgPool,
    customer_id: Uuid,
    application_id: Uuid,
    payment_product_id: Uuid,
    amount_cents: i64,
) -> Result<HomeLoanApplication, sqlx::Error> {
    let mut tx = db.begin().await?;

    let application = sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        SELECT id, customer_id, account_product_id, property_type, property_value_cents,
               down_payment_cents, loan_amount_cents, annual_rate_bps, term_years,
               monthly_payment_cents, outstanding_cents, status, reviewed_by, reviewed_at,
               created_at, updated_at
        FROM home_loan_applications
        WHERE id = $1 AND customer_id = $2 AND status = 'approved' AND outstanding_cents > 0
        FOR UPDATE
        "#,
    )
    .bind(application_id)
    .bind(customer_id)
    .fetch_one(&mut *tx)
    .await?;

    let product = lock_product_by_id(&mut tx, customer_id, payment_product_id).await?;
    let payment_cents = amount_cents.min(application.outstanding_cents);
    let new_product_balance = product.balance_cents - payment_cents;
    let new_outstanding = application.outstanding_cents - payment_cents;
    let new_status = if new_outstanding == 0 {
        "fully_paid"
    } else {
        "approved"
    };

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

    let updated = sqlx::query_as::<_, HomeLoanApplication>(
        r#"
        UPDATE home_loan_applications
        SET outstanding_cents = $1, status = $2, updated_at = NOW()
        WHERE id = $3
        RETURNING id, customer_id, account_product_id, property_type, property_value_cents,
                  down_payment_cents, loan_amount_cents, annual_rate_bps, term_years,
                  monthly_payment_cents, outstanding_cents, status, reviewed_by, reviewed_at,
                  created_at, updated_at
        "#,
    )
    .bind(new_outstanding)
    .bind(new_status)
    .bind(application.id)
    .fetch_one(&mut *tx)
    .await?;

    insert_product_transaction(
        &mut tx,
        product.id,
        customer_id,
        "home_loan_payment",
        payment_cents,
        new_product_balance,
        Some("Home loan repayment"),
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
    _customer_id: Uuid,
    transaction_type: &str,
    amount_cents: i64,
    balance_after_cents: i64,
    description: Option<&str>,
) -> Result<Transaction, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (product_id, transaction_type, amount_cents, balance_after_cents, description)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, product_id, transaction_type, amount_cents, balance_after_cents, description, created_at
        "#,
    )
    .bind(product_id)
    .bind(transaction_type)
    .bind(amount_cents)
    .bind(balance_after_cents)
    .bind(description)
    .fetch_one(&mut **tx)
    .await
}
