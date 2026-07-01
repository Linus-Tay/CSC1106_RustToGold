// Repository layer: isolates SQLx queries so services do not depend on raw database code.

use crate::models::{FixedDeposit, FixedDepositAdminRecord, FixedDepositPlan, Product, Transaction};
use chrono::NaiveDate;
use sqlx::{PgPool, Postgres, Transaction as DbTransaction};
use uuid::Uuid;

// Reads list active plans data from the database.
pub async fn list_active_plans(db: &PgPool) -> Result<Vec<FixedDepositPlan>, sqlx::Error> {
    sqlx::query_as::<_, FixedDepositPlan>(
        r#"
        SELECT id, plan_name, tenure_months, annual_rate_bps, minimum_amount_cents,
               is_active, created_at, updated_at
        FROM fixed_deposit_plans
        WHERE is_active = TRUE
        ORDER BY tenure_months ASC, annual_rate_bps DESC
        "#,
    )
    .fetch_all(db)
    .await
}

// Reads list all plans data from the database.
pub async fn list_all_plans(db: &PgPool) -> Result<Vec<FixedDepositPlan>, sqlx::Error> {
    sqlx::query_as::<_, FixedDepositPlan>(
        r#"
        SELECT id, plan_name, tenure_months, annual_rate_bps, minimum_amount_cents,
               is_active, created_at, updated_at
        FROM fixed_deposit_plans
        ORDER BY is_active DESC, tenure_months ASC, id ASC
        "#,
    )
    .fetch_all(db)
    .await
}

// Reads find plan by id data from the database.
pub async fn find_plan_by_id(db: &PgPool, plan_id: i64) -> Result<FixedDepositPlan, sqlx::Error> {
    sqlx::query_as::<_, FixedDepositPlan>(
        r#"
        SELECT id, plan_name, tenure_months, annual_rate_bps, minimum_amount_cents,
               is_active, created_at, updated_at
        FROM fixed_deposit_plans
        WHERE id = $1
        "#,
    )
    .bind(plan_id)
    .fetch_one(db)
    .await
}

// Reads list fixed deposits by customer data from the database.
pub async fn list_fixed_deposits_by_customer(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<Vec<FixedDeposit>, sqlx::Error> {
    sqlx::query_as::<_, FixedDeposit>(
        r#"
        SELECT fd.id, fd.customer_id, fd.funding_product_id, fd.plan_id,
               COALESCE(p.plan_name, 'Fixed Deposit Plan') AS plan_name,
               fd.principal_cents, fd.annual_rate_bps, fd.tenure_months, fd.interest_cents,
               fd.maturity_date, fd.status, fd.created_at, fd.updated_at
        FROM fixed_deposits fd
        LEFT JOIN fixed_deposit_plans p ON p.id = fd.plan_id
        WHERE fd.customer_id = $1
        ORDER BY fd.created_at DESC
        "#,
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
}

// Reads list all fixed deposit records data from the database.
pub async fn list_all_fixed_deposit_records(
    db: &PgPool,
) -> Result<Vec<FixedDepositAdminRecord>, sqlx::Error> {
    sqlx::query_as::<_, FixedDepositAdminRecord>(
        r#"
        SELECT fd.id,
               c.full_name AS customer_name,
               c.email AS customer_email,
               c.phone_number AS customer_phone,
               c.nric AS customer_nric,
               cp.account_number,
               cp.balance_cents AS account_balance_cents,
               COALESCE(p.plan_name, 'Fixed Deposit Plan') AS plan_name,
               fd.principal_cents,
               fd.annual_rate_bps,
               fd.tenure_months,
               fd.interest_cents,
               fd.maturity_date,
               fd.status,
               fd.created_at
        FROM fixed_deposits fd
        JOIN customers c ON c.id = fd.customer_id
        JOIN customer_products cp ON cp.id = fd.funding_product_id
        LEFT JOIN fixed_deposit_plans p ON p.id = fd.plan_id
        ORDER BY fd.created_at DESC
        "#,
    )
    .fetch_all(db)
    .await
}

// Persists the create fixed deposit database change.
pub async fn create_fixed_deposit(
    db: &PgPool,
    customer_id: Uuid,
    product_id: Uuid,
    plan: &FixedDepositPlan,
    principal_cents: i64,
    interest_cents: i64,
    maturity_date: NaiveDate,
) -> Result<FixedDeposit, sqlx::Error> {
    let mut tx = db.begin().await?;
    let product = lock_product_by_id(&mut tx, customer_id, product_id).await?;
    let new_balance = product.balance_cents - principal_cents;

    sqlx::query(
        r#"
        UPDATE customer_products
        SET balance_cents = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(new_balance)
    .bind(product.id)
    .execute(&mut *tx)
    .await?;

    let fd = sqlx::query_as::<_, FixedDeposit>(
        r#"
        INSERT INTO fixed_deposits (
            customer_id, funding_product_id, plan_id, principal_cents, annual_rate_bps,
            tenure_months, interest_cents, maturity_date, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'active')
        RETURNING id, customer_id, funding_product_id, plan_id,
                  $9::TEXT AS plan_name,
                  principal_cents, annual_rate_bps, tenure_months, interest_cents,
                  maturity_date, status, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .bind(product.id)
    .bind(plan.id)
    .bind(principal_cents)
    .bind(plan.annual_rate_bps)
    .bind(plan.tenure_months)
    .bind(interest_cents)
    .bind(maturity_date)
    .bind(&plan.plan_name)
    .fetch_one(&mut *tx)
    .await?;

    insert_product_transaction(
        &mut tx,
        product.id,
        customer_id,
        "fixed_deposit_open",
        principal_cents,
        new_balance,
        Some("Fixed deposit placement opened"),
    )
    .await?;

    tx.commit().await?;
    Ok(fd)
}

// Executes the database operation for mark customer matured.
pub async fn mark_customer_matured(db: &PgPool, customer_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE fixed_deposits
        SET status = 'matured', updated_at = NOW()
        WHERE customer_id = $1 AND status = 'active' AND maturity_date <= CURRENT_DATE
        "#,
    )
    .bind(customer_id)
    .execute(db)
    .await?;
    Ok(())
}

// Executes the database operation for withdraw fixed deposit.
pub async fn withdraw_fixed_deposit(
    db: &PgPool,
    customer_id: Uuid,
    fixed_deposit_id: Uuid,
) -> Result<String, sqlx::Error> {
    let mut tx = db.begin().await?;

    let fd = sqlx::query_as::<_, FixedDeposit>(
        r#"
        SELECT fd.id, fd.customer_id, fd.funding_product_id, fd.plan_id,
               COALESCE(p.plan_name, 'Fixed Deposit Plan') AS plan_name,
               fd.principal_cents, fd.annual_rate_bps, fd.tenure_months, fd.interest_cents,
               fd.maturity_date, fd.status, fd.created_at, fd.updated_at
        FROM fixed_deposits fd
        LEFT JOIN fixed_deposit_plans p ON p.id = fd.plan_id
        WHERE fd.id = $1 AND fd.customer_id = $2
        FOR UPDATE
        "#,
    )
    .bind(fixed_deposit_id)
    .bind(customer_id)
    .fetch_one(&mut *tx)
    .await?;

    let product = lock_product_by_id(&mut tx, customer_id, fd.funding_product_id).await?;
    let is_matured = fd.status == "matured" || (fd.status == "active" && fd.maturity_date <= chrono::Utc::now().date_naive());
    let payout_cents = if is_matured {
        fd.principal_cents + fd.interest_cents
    } else {
        fd.principal_cents
    };
    let new_status = if is_matured { "paid_out" } else { "withdrawn" };
    let transaction_type = if is_matured { "fixed_deposit_payout" } else { "fixed_deposit_withdrawal" };
    let description = if is_matured {
        "Matured fixed deposit payout"
    } else {
        "Early fixed deposit withdrawal"
    };
    let new_balance = product.balance_cents + payout_cents;

    sqlx::query(
        r#"
        UPDATE customer_products
        SET balance_cents = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(new_balance)
    .bind(product.id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE fixed_deposits
        SET status = $1, updated_at = NOW()
        WHERE id = $2 AND status IN ('active', 'matured')
        "#,
    )
    .bind(new_status)
    .bind(fd.id)
    .execute(&mut *tx)
    .await?;

    insert_product_transaction(
        &mut tx,
        product.id,
        customer_id,
        transaction_type,
        payout_cents,
        new_balance,
        Some(description),
    )
    .await?;

    tx.commit().await?;
    Ok(new_status.to_string())
}

// Persists the create plan database change.
pub async fn create_plan(
    db: &PgPool,
    plan_name: &str,
    tenure_months: i32,
    annual_rate_bps: i32,
    minimum_amount_cents: i64,
    is_active: bool,
) -> Result<FixedDepositPlan, sqlx::Error> {
    sqlx::query_as::<_, FixedDepositPlan>(
        r#"
        INSERT INTO fixed_deposit_plans (plan_name, tenure_months, annual_rate_bps, minimum_amount_cents, is_active)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, plan_name, tenure_months, annual_rate_bps, minimum_amount_cents, is_active, created_at, updated_at
        "#,
    )
    .bind(plan_name)
    .bind(tenure_months)
    .bind(annual_rate_bps)
    .bind(minimum_amount_cents)
    .bind(is_active)
    .fetch_one(db)
    .await
}

// Persists the update plan database change.
pub async fn update_plan(
    db: &PgPool,
    plan_id: i64,
    plan_name: &str,
    tenure_months: i32,
    annual_rate_bps: i32,
    minimum_amount_cents: i64,
    is_active: bool,
) -> Result<FixedDepositPlan, sqlx::Error> {
    sqlx::query_as::<_, FixedDepositPlan>(
        r#"
        UPDATE fixed_deposit_plans
        SET plan_name = $1,
            tenure_months = $2,
            annual_rate_bps = $3,
            minimum_amount_cents = $4,
            is_active = $5,
            updated_at = NOW()
        WHERE id = $6
        RETURNING id, plan_name, tenure_months, annual_rate_bps, minimum_amount_cents, is_active, created_at, updated_at
        "#,
    )
    .bind(plan_name)
    .bind(tenure_months)
    .bind(annual_rate_bps)
    .bind(minimum_amount_cents)
    .bind(is_active)
    .bind(plan_id)
    .fetch_one(db)
    .await
}

// Persists the lock product by id database change.
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

// Persists the insert product transaction database change.
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
