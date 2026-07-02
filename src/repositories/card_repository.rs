// Repository layer: isolates SQLx queries so services do not depend on raw database code.

use crate::models::Card;
use sqlx::PgPool;
use uuid::Uuid;

// Reads list cards by customer data from the database.
pub async fn list_cards_by_customer(db: &PgPool, customer_id: Uuid) -> Result<Vec<Card>, sqlx::Error> {
    sqlx::query_as::<_, Card>(
        r#"
        SELECT c.id, c.customer_id, c.linked_product_id, cp.account_number,
               c.pin_hash, c.card_type, c.display_name, c.card_number, c.status, c.created_at, c.updated_at
        FROM cards c
        JOIN customer_products cp ON cp.id = c.linked_product_id
        WHERE c.customer_id = $1
        ORDER BY c.created_at DESC
        "#,
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
}

// Find active card by linked product
pub async fn find_card_by_linked_account(db: &PgPool, linked_product_id: Uuid) -> Result<Option<Card>, sqlx::Error> {
    sqlx::query_as::<_, Card>(
        r#"
        SELECT c.id, c.customer_id, c.linked_product_id, cp.account_number,
               c.card_type, c.pin_hash, c.display_name, c.card_number, c.status, c.created_at, c.updated_at
        FROM cards c
        JOIN customer_products cp ON cp.id = c.linked_product_id
        WHERE cp.id = $1
        "#,
    )
    .bind(linked_product_id)
    .fetch_optional(db)
    .await
}

// Reads list cards by customer data from the database.
pub async fn find_active_by_card_number(db: &PgPool, card_number: &str) -> Result<Option<Card>, sqlx::Error> {
    sqlx::query_as::<_, Card>(
        r#"
        SELECT c.id, c.customer_id, c.linked_product_id, cp.account_number,
               c.card_type, c.pin_hash, c.display_name, c.card_number, c.status, c.created_at, c.updated_at
        FROM cards c
        JOIN customer_products cp ON cp.id = c.linked_product_id
        WHERE c.card_number = $1
        AND c.status = 'active'
        "#,
    )
    .bind(card_number)
    .fetch_optional(db)
    .await
}

// Reads list cards by customer data from the database.
pub async fn find_active_by_card_id(db: &PgPool, card_id: &Uuid) -> Result<Option<Card>, sqlx::Error> {
    sqlx::query_as::<_, Card>(
        r#"
        SELECT c.id, c.customer_id, c.linked_product_id, cp.account_number,
               c.card_type, c.pin_hash, c.display_name, c.card_number, c.status, c.created_at, c.updated_at
        FROM cards c
        JOIN customer_products cp ON cp.id = c.linked_product_id
        WHERE c.id = $1
        AND c.status = 'active'
        "#,
    )
    .bind(card_id)
    .fetch_optional(db)
    .await
}

// Persists the create card database change.
pub async fn create_card(
    db: &PgPool,
    customer_id: Uuid,
    linked_product_id: Uuid,
    card_type: &str,
    pin_hash: &str,
    display_name: &str,
    card_number: &str,
) -> Result<Card, sqlx::Error> {
    sqlx::query_as::<_, Card>(
        r#"
        INSERT INTO cards (customer_id, linked_product_id, card_type, pin_hash, display_name, card_number, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'active')
        RETURNING id, customer_id, linked_product_id,
            (SELECT account_number FROM customer_products WHERE id = $2) AS account_number,
            pin_hash, card_type, display_name, card_number, status, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .bind(linked_product_id)
    .bind(card_type)
    .bind(pin_hash)
    .bind(display_name)
    .bind(card_number)
    .fetch_one(db)
    .await
}

// Persists the set card status database change.
pub async fn set_card_status(
    db: &PgPool,
    customer_id: Uuid,
    card_id: Uuid,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE cards
        SET status = $1, updated_at = NOW()
        WHERE id = $2 AND customer_id = $3
        "#,
    )
    .bind(status)
    .bind(card_id)
    .bind(customer_id)
    .execute(db)
    .await?;

    Ok(())
}
