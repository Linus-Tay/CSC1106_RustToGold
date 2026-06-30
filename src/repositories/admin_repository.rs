use crate::models::{
    AdminCustomerApplication, AdminDashboardSummary, AdminHomeLoanRecord, AdminPersonalLoanRecord,
    Product,
};
use sqlx::{PgPool, Postgres, Transaction as DbTransaction};
use uuid::Uuid;

pub async fn dashboard_summary(db: &PgPool) -> Result<AdminDashboardSummary, sqlx::Error> {
    sqlx::query_as::<_, AdminDashboardSummary>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM customers WHERE kyc_status = 'pending')::BIGINT AS pending_signup_count,
            (SELECT COUNT(*) FROM personal_loans WHERE status = 'pending')::BIGINT AS pending_personal_loan_count,
            (SELECT COUNT(*) FROM home_loan_applications WHERE status = 'pending')::BIGINT AS pending_home_loan_count,
            (SELECT COUNT(*) FROM fixed_deposits WHERE status IN ('active', 'matured'))::BIGINT AS active_fixed_deposit_count,
            (SELECT COUNT(*) FROM customers)::BIGINT AS total_customer_count
        "#,
    )
    .fetch_one(db)
    .await
}

pub async fn list_customer_applications(
    db: &PgPool,
) -> Result<Vec<AdminCustomerApplication>, sqlx::Error> {
    sqlx::query_as::<_, AdminCustomerApplication>(
        r#"
        SELECT
            c.id AS customer_id,
            u.id AS user_id,
            c.full_name,
            c.email,
            c.phone_number,
            c.nric,
            c.date_of_birth,
            c.gender,
            c.nationality,
            c.residency,
            c.race,
            c.residential_address,
            c.mailing_address,
            c.preferred_contact,
            c.employment_status,
            c.occupation,
            c.employer_name,
            c.industry,
            c.monthly_income_range,
            c.kyc_status,
            u.status AS user_status,
            cp.account_number,
            cp.product_id AS selected_account_type,
            cp.product_type,
            cp.status AS product_status,
            cp.balance_cents AS account_balance_cents,
            cp.created_at AS account_created_at,
            c.created_at
        FROM customers c
        JOIN users u ON u.customer_id = c.id
        LEFT JOIN LATERAL (
            SELECT account_number, product_id, product_type, status, balance_cents, created_at
            FROM customer_products
            WHERE customer_id = c.id
            ORDER BY created_at ASC
            LIMIT 1
        ) cp ON TRUE
        ORDER BY
            CASE c.kyc_status WHEN 'pending' THEN 0 WHEN 'approved' THEN 1 ELSE 2 END,
            c.created_at DESC
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn approve_customer_application(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        UPDATE customers
        SET kyc_status = 'approved', updated_at = NOW()
        WHERE id = $1 AND kyc_status = 'pending'
        "#,
    )
    .bind(customer_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE customer_products
        SET status = 'active', updated_at = NOW()
        WHERE customer_id = $1 AND status <> 'closed'
        "#,
    )
    .bind(customer_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE users
        SET status = 'active', updated_at = NOW()
        WHERE customer_id = $1
        "#,
    )
    .bind(customer_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await
}

pub async fn reject_customer_application(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        UPDATE customers
        SET kyc_status = 'rejected', updated_at = NOW()
        WHERE id = $1 AND kyc_status = 'pending'
        "#,
    )
    .bind(customer_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE customer_products
        SET status = 'inactive', updated_at = NOW()
        WHERE customer_id = $1 AND status = 'active'
        "#,
    )
    .bind(customer_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await
}

pub async fn list_personal_loans(
    db: &PgPool,
) -> Result<Vec<AdminPersonalLoanRecord>, sqlx::Error> {
    sqlx::query_as::<_, AdminPersonalLoanRecord>(personal_loan_select_sql())
        .fetch_all(db)
        .await
}

pub async fn list_home_loans(db: &PgPool) -> Result<Vec<AdminHomeLoanRecord>, sqlx::Error> {
    sqlx::query_as::<_, AdminHomeLoanRecord>(
        r#"
        SELECT
            hl.id,
            hl.customer_id,
            c.full_name AS customer_name,
            c.email AS customer_email,
            c.phone_number AS customer_phone,
            c.nric AS customer_nric,
            c.kyc_status,
            c.employment_status,
            c.occupation,
            c.employer_name,
            c.monthly_income_range,
            cp.account_number,
            cp.balance_cents AS account_balance_cents,
            cp.status AS account_status,
            hl.property_type,
            hl.property_value_cents,
            hl.down_payment_cents,
            hl.loan_amount_cents,
            hl.annual_rate_bps,
            hl.term_years,
            hl.monthly_payment_cents,
            hl.outstanding_cents,
            hl.status,
            hl.created_at
        FROM home_loan_applications hl
        JOIN customers c ON c.id = hl.customer_id
        LEFT JOIN customer_products cp ON cp.id = hl.account_product_id
        ORDER BY
            CASE hl.status WHEN 'pending' THEN 0 WHEN 'approved' THEN 1 ELSE 2 END,
            hl.created_at DESC
        "#,
    )
    .fetch_all(db)
    .await
}

pub async fn approve_personal_loan(
    db: &PgPool,
    staff_user_id: i64,
    loan_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    let loan_query = format!(
        "{} WHERE pl.id = $1 AND pl.status = 'pending' FOR UPDATE OF pl",
        personal_loan_select_base_sql()
    );

    let loan = sqlx::query_as::<_, AdminPersonalLoanRecord>(&loan_query)
        .bind(loan_id)
        .fetch_one(&mut *tx)
        .await?;

    let product = lock_customer_product(&mut tx, loan.customer_id).await?;
    let new_balance = product.balance_cents + loan.principal_cents;

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
        UPDATE personal_loans
        SET status = 'active',
            outstanding_cents = principal_cents,
            reviewed_by = $1,
            reviewed_at = NOW(),
            updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(staff_user_id)
    .bind(loan.id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO transactions (product_id, customer_id, transaction_type, amount_cents, balance_after_cents, description)
        VALUES ($1, $2, 'loan_disbursement', $3, $4, 'Personal loan approved and disbursed')
        "#,
    )
    .bind(product.id)
    .bind(loan.customer_id)
    .bind(loan.principal_cents)
    .bind(new_balance)
    .execute(&mut *tx)
    .await?;

    tx.commit().await
}

pub async fn reject_personal_loan(
    db: &PgPool,
    staff_user_id: i64,
    loan_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE personal_loans
        SET status = 'rejected', reviewed_by = $1, reviewed_at = NOW(), updated_at = NOW()
        WHERE id = $2 AND status = 'pending'
        "#,
    )
    .bind(staff_user_id)
    .bind(loan_id)
    .execute(db)
    .await?;

    Ok(())
}

async fn lock_customer_product(
    tx: &mut DbTransaction<'_, Postgres>,
    customer_id: Uuid,
) -> Result<Product, sqlx::Error> {
    sqlx::query_as::<_, Product>(
        r#"
        SELECT id, customer_id, account_number, product_id, product_type, balance_cents, status, created_at, updated_at
        FROM customer_products
        WHERE customer_id = $1 AND status = 'active'
        ORDER BY created_at ASC
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(customer_id)
    .fetch_one(&mut **tx)
    .await
}

fn personal_loan_select_sql() -> &'static str {
    r#"
    SELECT * FROM (
        SELECT
            pl.id,
            pl.customer_id,
            c.full_name AS customer_name,
            c.email AS customer_email,
            c.phone_number AS customer_phone,
            c.nric AS customer_nric,
            c.kyc_status,
            c.employment_status,
            c.occupation,
            c.employer_name,
            c.monthly_income_range,
            cp.account_number,
            cp.balance_cents AS account_balance_cents,
            cp.status AS account_status,
            pl.purpose,
            pl.principal_cents,
            pl.annual_rate_bps,
            pl.term_months,
            pl.monthly_payment_cents,
            pl.outstanding_cents,
            pl.status,
            pl.created_at
        FROM personal_loans pl
        JOIN customers c ON c.id = pl.customer_id
        LEFT JOIN customer_products cp ON cp.id = pl.funding_product_id
    ) personal_loan_records
    ORDER BY
        CASE status WHEN 'pending' THEN 0 WHEN 'active' THEN 1 ELSE 2 END,
        created_at DESC
    "#
}

fn personal_loan_select_base_sql() -> &'static str {
    r#"
    SELECT
        pl.id,
        pl.customer_id,
        c.full_name AS customer_name,
        c.email AS customer_email,
        c.phone_number AS customer_phone,
        c.nric AS customer_nric,
        c.kyc_status,
        c.employment_status,
        c.occupation,
        c.employer_name,
        c.monthly_income_range,
        cp.account_number,
        cp.balance_cents AS account_balance_cents,
        cp.status AS account_status,
        pl.purpose,
        pl.principal_cents,
        pl.annual_rate_bps,
        pl.term_months,
        pl.monthly_payment_cents,
        pl.outstanding_cents,
        pl.status,
        pl.created_at
    FROM personal_loans pl
    JOIN customers c ON c.id = pl.customer_id
    LEFT JOIN customer_products cp ON cp.id = pl.funding_product_id
    "#
}
