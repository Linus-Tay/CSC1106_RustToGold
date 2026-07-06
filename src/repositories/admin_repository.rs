
use crate::models::{
    AdminAuditLogRecord, AdminCustomerAccountRecord, AdminCustomerApplication, AdminDashboardSummary,
    AdminHomeLoanRecord, AdminPersonalLoanRecord, AdminStaffUser, Product,
};
use sqlx::{PgPool, Postgres, Transaction as DbTransaction};
use uuid::Uuid;

// Query dashboard summary
pub async fn dashboard_summary(db: &PgPool) -> Result<AdminDashboardSummary, sqlx::Error> {
    sqlx::query_as::<_, AdminDashboardSummary>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM customers WHERE kyc_status = 'pending')::BIGINT AS pending_signup_count,
            (SELECT COUNT(*)
             FROM customer_products cp
             JOIN customers c ON c.id = cp.customer_id
             WHERE cp.status = 'inactive' AND c.kyc_status = 'approved')::BIGINT AS pending_account_product_count,
            (SELECT COUNT(*) FROM personal_loans WHERE status = 'pending')::BIGINT AS pending_personal_loan_count,
            (SELECT COUNT(*) FROM home_loan_applications WHERE status = 'pending')::BIGINT AS pending_home_loan_count,
            (SELECT COUNT(*) FROM fraud_alerts WHERE rule_code IN ('HIGH_VALUE_MONITORING', 'HIGH_VALUE_REVIEW') AND status IN ('blocked', 'flagged', 'reviewed'))::BIGINT AS high_value_alert_count,
            (SELECT COUNT(*) FROM fixed_deposits WHERE status IN ('active', 'matured'))::BIGINT AS active_fixed_deposit_count,
            (SELECT COUNT(*) FROM customers)::BIGINT AS total_customer_count
        "#,
    )
    .fetch_one(db)
    .await
}

// Query list customer applications
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
        LEFT JOIN users u ON u.customer_id = c.id AND u.role = 'customer'
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

// Persist approve customer application
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

// Persist reject customer application
pub async fn reject_customer_application(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        UPDATE customers
        SET kyc_status = 'rejected',
            nric = CONCAT('REJECTED-', REPLACE(id::text, '-', '')),
            email = CONCAT('rejected+', REPLACE(id::text, '-', ''), '@rusttogold.local'),
            updated_at = NOW()
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

// Query list personal loans
pub async fn list_personal_loans(
    db: &PgPool,
) -> Result<Vec<AdminPersonalLoanRecord>, sqlx::Error> {
    sqlx::query_as::<_, AdminPersonalLoanRecord>(personal_loan_select_sql())
        .fetch_all(db)
        .await
}

// Query list home loans
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

// Persist approve personal loan
pub async fn approve_personal_loan(
    db: &PgPool,
    staff_user_id: Uuid,
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

    let product = lock_customer_product_by_id(&mut tx, loan.customer_id, loan.funding_product_id).await?;
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
        INSERT INTO transactions (product_id, transaction_type, amount_cents, balance_after_cents, description)
        VALUES ($1, 'loan_disbursement', $2, $3, 'Personal loan approved and disbursed')
        "#,
    )
    .bind(product.id)
    .bind(loan.principal_cents)
    .bind(new_balance)
    .execute(&mut *tx)
    .await?;

    insert_audit_log_tx(
        &mut tx,
        Some(staff_user_id),
        "approve_personal_loan",
        "personal_loan",
        Some(loan.id.to_string()),
        Some(format!("Approved and disbursed {} to account {}", loan.principal_display(), product.account_number)),
    )
    .await?;

    tx.commit().await
}

// Persist reject personal loan
pub async fn reject_personal_loan(
    db: &PgPool,
    staff_user_id: Uuid,
    loan_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        UPDATE personal_loans
        SET status = 'rejected', reviewed_by = $1, reviewed_at = NOW(), updated_at = NOW()
        WHERE id = $2 AND status = 'pending'
        "#,
    )
    .bind(staff_user_id)
    .bind(loan_id)
    .execute(&mut *tx)
    .await?;

    insert_audit_log_tx(
        &mut tx,
        Some(staff_user_id),
        "reject_personal_loan",
        "personal_loan",
        Some(loan_id.to_string()),
        Some("Personal loan application rejected".to_string()),
    )
    .await?;

    tx.commit().await
}

// Query record audit log
pub async fn record_audit_log(
    db: &PgPool,
    actor_user_id: Option<Uuid>,
    action: &str,
    entity_type: &str,
    entity_id: Option<String>,
    details: Option<String>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs (actor_user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(actor_user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(details)
    .execute(db)
    .await?;

    Ok(())
}

// Query list staff users
pub async fn list_staff_users(db: &PgPool) -> Result<Vec<AdminStaffUser>, sqlx::Error> {
    sqlx::query_as::<_, AdminStaffUser>(
        r#"
        SELECT id, username, full_name, email, phone_number, role, status, last_login_at, created_at
        FROM users
        WHERE role IN ('staff', 'admin')
        ORDER BY CASE role WHEN 'admin' THEN 0 ELSE 1 END, created_at DESC
        "#,
    )
    .fetch_all(db)
    .await
}

// Persist create staff user
pub async fn create_staff_user(
    db: &PgPool,
    username: &str,
    full_name: &str,
    email: &str,
    phone_number: &str,
    role: &str,
    password_hash: &str,
    actor_user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    let staff_user_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO users (username, full_name, email, phone_number, password_hash, role, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'active')
        RETURNING id
        "#,
    )
    .bind(username)
    .bind(full_name)
    .bind(email)
    .bind(phone_number)
    .bind(password_hash)
    .bind(role)
    .fetch_one(&mut *tx)
    .await?;

    insert_audit_log_tx(
        &mut tx,
        Some(actor_user_id),
        "create_staff_user",
        "user",
        Some(staff_user_id.to_string()),
        Some(format!("Created {role} user {username}")),
    )
    .await?;

    tx.commit().await
}

// Persist update staff user
pub async fn update_staff_user(
    db: &PgPool,
    staff_user_id: Uuid,
    full_name: &str,
    email: &str,
    phone_number: &str,
    role: &str,
    status: &str,
    password_hash: Option<&str>,
    actor_user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    if let Some(hash) = password_hash {
        sqlx::query(
            r#"
            UPDATE users
            SET full_name = $1, email = $2, phone_number = $3, role = $4, status = $5, password_hash = $6, updated_at = NOW()
            WHERE id = $7 AND role IN ('staff', 'admin')
            "#,
        )
        .bind(full_name)
        .bind(email)
        .bind(phone_number)
        .bind(role)
        .bind(status)
        .bind(hash)
        .bind(staff_user_id)
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query(
            r#"
            UPDATE users
            SET full_name = $1, email = $2, phone_number = $3, role = $4, status = $5, updated_at = NOW()
            WHERE id = $6 AND role IN ('staff', 'admin')
            "#,
        )
        .bind(full_name)
        .bind(email)
        .bind(phone_number)
        .bind(role)
        .bind(status)
        .bind(staff_user_id)
        .execute(&mut *tx)
        .await?;
    }

    insert_audit_log_tx(
        &mut tx,
        Some(actor_user_id),
        "update_staff_user",
        "user",
        Some(staff_user_id.to_string()),
        Some(format!("Updated staff/admin user details; status={status}")),
    )
    .await?;

    tx.commit().await
}

// Persist delete staff user
pub async fn delete_staff_user(
    db: &PgPool,
    staff_user_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    sqlx::query("DELETE FROM users WHERE id = $1 AND role = 'staff'")
        .bind(staff_user_id)
        .execute(&mut *tx)
        .await?;

    insert_audit_log_tx(
        &mut tx,
        Some(actor_user_id),
        "delete_staff_user",
        "user",
        Some(staff_user_id.to_string()),
        Some("Deleted staff user".to_string()),
    )
    .await?;

    tx.commit().await
}

// Query list customer accounts
pub async fn list_customer_accounts(db: &PgPool) -> Result<Vec<AdminCustomerAccountRecord>, sqlx::Error> {
    sqlx::query_as::<_, AdminCustomerAccountRecord>(
        r#"
        SELECT
            cp.id AS product_id,
            cp.customer_id,
            c.full_name AS customer_name,
            c.email AS customer_email,
            c.kyc_status AS customer_kyc_status,
            u.id AS user_id,
            u.username,
            u.status AS user_status,
            cp.account_number,
            cp.product_id AS account_product_id,
            cp.product_type,
            cp.status AS product_status,
            cp.balance_cents,
            cp.created_at
        FROM customer_products cp
        JOIN customers c ON c.id = cp.customer_id
        LEFT JOIN users u ON u.customer_id = c.id AND u.role = 'customer'
        ORDER BY CASE cp.status WHEN 'inactive' THEN 0 WHEN 'active' THEN 1 WHEN 'frozen' THEN 2 ELSE 3 END, cp.created_at DESC
        "#,
    )
    .fetch_all(db)
    .await
}

// Persist set user status
pub async fn set_user_status(
    db: &PgPool,
    target_user_id: Uuid,
    status: &str,
    actor_user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        UPDATE users
        SET status = $1, updated_at = NOW()
        WHERE id = $2 AND role = 'customer'
        "#,
    )
    .bind(status)
    .bind(target_user_id)
    .execute(&mut *tx)
    .await?;

    insert_audit_log_tx(
        &mut tx,
        Some(actor_user_id),
        "set_customer_user_status",
        "user",
        Some(target_user_id.to_string()),
        Some(format!("Customer online banking status changed to {status}")),
    )
    .await?;

    tx.commit().await
}

// Persist set product status
pub async fn set_product_status(
    db: &PgPool,
    product_id: Uuid,
    status: &str,
    actor_user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        UPDATE customer_products
        SET status = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(status)
    .bind(product_id)
    .execute(&mut *tx)
    .await?;

    insert_audit_log_tx(
        &mut tx,
        Some(actor_user_id),
        "set_product_status",
        "customer_product",
        Some(product_id.to_string()),
        Some(format!("Customer product status changed to {status}")),
    )
    .await?;

    tx.commit().await
}

// Query list audit logs
pub async fn list_audit_logs(db: &PgPool) -> Result<Vec<AdminAuditLogRecord>, sqlx::Error> {
    sqlx::query_as::<_, AdminAuditLogRecord>(
        r#"
        SELECT
            al.id,
            al.actor_user_id,
            u.username AS actor_username,
            al.action,
            al.entity_type,
            al.entity_id,
            al.details,
            al.created_at
        FROM audit_logs al
        LEFT JOIN users u ON u.id = al.actor_user_id
        ORDER BY al.created_at DESC
        LIMIT 200
        "#,
    )
    .fetch_all(db)
    .await
}

// Persist insert audit log tx
async fn insert_audit_log_tx(
    tx: &mut DbTransaction<'_, Postgres>,
    actor_user_id: Option<Uuid>,
    action: &str,
    entity_type: &str,
    entity_id: Option<String>,
    details: Option<String>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs (actor_user_id, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(actor_user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(details)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

// Persist lock customer product by id
async fn lock_customer_product_by_id(
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

// Persist lock customer product
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

// Query personal loan select sql
fn personal_loan_select_sql() -> &'static str {
    r#"
    SELECT * FROM (
        SELECT
            pl.id,
            pl.customer_id,
            pl.funding_product_id,
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

// Query personal loan select base sql
fn personal_loan_select_base_sql() -> &'static str {
    r#"
    SELECT
        pl.id,
        pl.customer_id,
        pl.funding_product_id,
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
