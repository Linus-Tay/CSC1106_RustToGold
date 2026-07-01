use crate::models::GiroArrangement;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

const GIRO_SELECT: &str = r#"
    SELECT ga.id, ga.customer_id, ga.from_product_id, from_cp.account_number,
           ga.recipient_product_id, recipient_cp.account_number AS recipient_account_number,
           recipient_cp.product_id AS recipient_account_label, ga.payee_name, ga.amount_cents,
           ga.frequency, ga.next_payment_date, ga.end_date, ga.note, ga.status,
           ga.created_at, ga.updated_at
    FROM giro_arrangements ga
    JOIN customer_products from_cp ON from_cp.id = ga.from_product_id
    JOIN customer_products recipient_cp ON recipient_cp.id = ga.recipient_product_id
"#;

pub async fn list_by_customer(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<Vec<GiroArrangement>, sqlx::Error> {
    let query = format!(
        "{} WHERE ga.customer_id = $1 ORDER BY CASE ga.status WHEN 'active' THEN 0 ELSE 1 END, ga.next_payment_date ASC",
        GIRO_SELECT
    );

    sqlx::query_as::<_, GiroArrangement>(&query)
        .bind(customer_id)
        .fetch_all(db)
        .await
}

pub async fn insert_arrangement(
    db: &PgPool,
    customer_id: Uuid,
    from_product_id: Uuid,
    recipient_product_id: Uuid,
    payee_name: &str,
    amount_cents: i64,
    frequency: &str,
    next_payment_date: NaiveDate,
    end_date: Option<NaiveDate>,
    note: Option<&str>,
) -> Result<GiroArrangement, sqlx::Error> {
    let query = format!(
        r#"
        WITH inserted AS (
            INSERT INTO giro_arrangements
                (customer_id, from_product_id, recipient_product_id, payee_name, amount_cents, frequency, next_payment_date, end_date, note, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active')
            RETURNING id
        )
        {}
        JOIN inserted ON inserted.id = ga.id
        "#,
        GIRO_SELECT
    );

    sqlx::query_as::<_, GiroArrangement>(&query)
        .bind(customer_id)
        .bind(from_product_id)
        .bind(recipient_product_id)
        .bind(payee_name)
        .bind(amount_cents)
        .bind(frequency)
        .bind(next_payment_date)
        .bind(end_date)
        .bind(note)
        .fetch_one(db)
        .await
}

pub async fn cancel_arrangement(
    db: &PgPool,
    customer_id: Uuid,
    arrangement_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE giro_arrangements
        SET status = 'cancelled', updated_at = NOW()
        WHERE id = $1 AND customer_id = $2 AND status = 'active'
        "#,
    )
    .bind(arrangement_id)
    .bind(customer_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}
