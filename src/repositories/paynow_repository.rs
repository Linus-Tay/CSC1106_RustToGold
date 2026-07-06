use crate::models::{PayNowRegistration, Product};
use sqlx::PgPool;
use uuid::Uuid;

// Query list by customer
pub async fn list_by_customer(
    db: &PgPool,
    customer_id: Uuid,
) -> Result<Vec<PayNowRegistration>, sqlx::Error> {
    sqlx::query_as::<_, PayNowRegistration>(
        r#"
        SELECT pr.id, pr.customer_id, pr.paynow_type, pr.paynow_id,
               pr.linked_account_id, cp.account_number, cp.product_id,
               cp.balance_cents, pr.status, pr.registered_at
        FROM registered_paynow pr
        JOIN customer_products cp ON cp.id = pr.linked_account_id
        WHERE pr.customer_id = $1 AND pr.status = 'active'
        ORDER BY
            CASE pr.status WHEN 'active' THEN 0 ELSE 1 END,
            pr.registered_at DESC
        "#,
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
}

// Query find active by identifier
pub async fn find_active_by_identifier(
    db: &PgPool,
    paynow_type: &str,
    paynow_id: &str
) -> Result<Option<PayNowRegistration>, sqlx::Error> {
    sqlx::query_as::<_, PayNowRegistration>(
        r#"
        SELECT pr.id, pr.customer_id, pr.paynow_type, pr.paynow_id,
               pr.linked_account_id, cp.account_number, cp.product_id,
               cp.balance_cents, pr.status, pr.registered_at
        FROM registered_paynow pr
        JOIN customer_products cp ON cp.id = pr.linked_account_id
        WHERE pr.paynow_type = $1
          AND lower(pr.paynow_id) = lower($2)
          AND pr.status = 'active'
        LIMIT 1
        "#,
    )
    .bind(paynow_type)
    .bind(paynow_id)
    .fetch_optional(db)
    .await
}

// Query find active by product id
pub async fn find_active_by_product_id(
    db: &PgPool,
    product_id: &uuid::Uuid
) -> Result<Option<PayNowRegistration>, sqlx::Error> {
    sqlx::query_as::<_, PayNowRegistration>(
        r#"
        SELECT pr.id, pr.customer_id, pr.paynow_type, pr.paynow_id,
               pr.linked_account_id, cp.account_number, cp.product_id,
               cp.balance_cents, pr.status, pr.registered_at
        FROM registered_paynow pr
        JOIN customer_products cp ON cp.id = pr.linked_account_id
        WHERE cp.id = $1
        AND pr.status = 'active'
        LIMIT 1
        "#,
    )
    .bind(product_id)
    .fetch_optional(db)
    .await
}

// Persist set paynow to inactive
pub async fn set_paynow_to_inactive(
    db: &PgPool,
    paynow_id: &Uuid
) -> Result<Option<PayNowRegistration>, sqlx::Error> {
    sqlx::query_as::<_, PayNowRegistration>(
        r#"
        WITH updated_paynow AS (
            UPDATE registered_paynow
            SET status = 'inactive'
            WHERE id = $1
            RETURNING *
        )
        SELECT 
            up.id, up.customer_id, up.paynow_type, up.paynow_id,
            up.linked_account_id, cp.account_number, cp.product_id,
            cp.balance_cents, up.status, up.registered_at
        FROM updated_paynow up
        JOIN customer_products cp ON cp.id = up.linked_account_id
        "#,
    )
    .bind(paynow_id)
    .fetch_optional(db)
    .await
}

// Persist insert registration
pub async fn insert_registration(
    db: &PgPool,
    customer_id: Uuid,
    paynow_type: &str,
    paynow_id: &str,
    linked_product_id: Uuid,
) -> Result<PayNowRegistration, sqlx::Error> {
    sqlx::query_as::<_, PayNowRegistration>(
        r#"
        WITH inserted AS (
            INSERT INTO registered_paynow (customer_id, paynow_type, paynow_id, linked_account_id, status)
            VALUES ($1, $2, $3, $4, 'active')
            RETURNING id, customer_id, paynow_type, paynow_id, linked_account_id, status, registered_at
        )
        SELECT inserted.id, inserted.customer_id, inserted.paynow_type, inserted.paynow_id,
               inserted.linked_account_id, cp.account_number, cp.product_id,
               cp.balance_cents, inserted.status, inserted.registered_at
        FROM inserted
        JOIN customer_products cp ON cp.id = inserted.linked_account_id
        "#,
    )
    .bind(customer_id)
    .bind(paynow_type)
    .bind(paynow_id)
    .bind(linked_product_id)
    .fetch_one(db)
    .await
}

// Query upsert phone registration
pub async fn upsert_phone_registration(
    db: &PgPool,
    customer_id: Uuid,
    paynow_id: &str,
    linked_product_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    let existing_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM registered_paynow
        WHERE customer_id = $1
          AND paynow_type = 'phone_number'
          AND status = 'active'
        ORDER BY registered_at DESC
        LIMIT 1
        "#,
    )
    .bind(customer_id)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(existing_id) = existing_id {
        sqlx::query(
            r#"
            UPDATE registered_paynow
            SET paynow_id = $1,
                linked_account_id = $2,
                status = 'active'
            WHERE id = $3
            "#,
        )
        .bind(paynow_id)
        .bind(linked_product_id)
        .bind(existing_id)
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO registered_paynow (customer_id, paynow_type, paynow_id, linked_account_id, status)
            VALUES ($1, 'phone_number', $2, $3, 'active')
            "#,
        )
        .bind(customer_id)
        .bind(paynow_id)
        .bind(linked_product_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

// Query execute paynow transfer
pub async fn execute_paynow_transfer(
    db: &PgPool,
    sender_customer_id: Uuid,
    sender_product_id: Uuid,
    recipient_type: &str,
    recipient_id: &str,
    amount_cents: i64,
    note: Option<&str>,
) -> Result<(bool, Option<String>), sqlx::Error> {
    if amount_cents <= 0 {
        return Ok((false, Some("Transfer amount must be greater than zero.".to_string())));
    }

    let mut tx = db.begin().await?;

    let recipient_info = sqlx::query!(
        r#"
        SELECT cp.id AS account_id, cp.customer_id
        FROM registered_paynow pr
        JOIN customer_products cp ON cp.id = pr.linked_account_id
        WHERE pr.paynow_type = $1 
          AND lower(pr.paynow_id) = lower($2)
          AND pr.status = 'active'
          AND cp.status = 'active'
        ORDER BY pr.registered_at DESC
        LIMIT 1
        "#,
        recipient_type,
        recipient_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(recipient) = recipient_info else {
        return Ok((false, Some("No active PayNow recipient was found.".to_string())));
    };

    if recipient.customer_id == sender_customer_id {
        return Ok((false, Some("You cannot transfer to your own PayNow registration.".to_string())));
    }

    let (first_lock_id, second_lock_id) = if sender_product_id < recipient.account_id {
        (sender_product_id, recipient.account_id)
    } else {
        (recipient.account_id, sender_product_id)
    };

    let locked_accounts = sqlx::query_as::<_, Product>(
        r#"
        SELECT id, customer_id, account_number, product_id, product_type,
               balance_cents, status, created_at, updated_at
        FROM customer_products
        WHERE id = ANY($1) AND status = 'active'
        ORDER BY id
        FOR UPDATE
        "#
    )
    .bind(&[first_lock_id, second_lock_id][..])
    .fetch_all(&mut *tx)
    .await?;

    if locked_accounts.len() != 2 {
        return Ok((false, Some("One or both accounts are unavailable.".to_string())));
    }

    let sender_product = locked_accounts.iter().find(|a| a.id == sender_product_id).unwrap();
    let recipient_product = locked_accounts.iter().find(|a| a.id == recipient.account_id).unwrap();

    if sender_product.customer_id != sender_customer_id {
        return Ok((false, Some("Choose an active account that belongs to you.".to_string())));
    }

    if sender_product.balance_cents < amount_cents {
        return Ok((false, Some("Insufficient funds in the selected account.".to_string())));
    }

    let sender_new_balance = sender_product.balance_cents - amount_cents;
    let recipient_new_balance = recipient_product.balance_cents + amount_cents;

    sqlx::query("UPDATE customer_products SET balance_cents = $1, updated_at = NOW() WHERE id = $2")
        .bind(sender_new_balance)
        .bind(sender_product.id)
        .execute(&mut *tx).await?;

    sqlx::query("UPDATE customer_products SET balance_cents = $1, updated_at = NOW() WHERE id = $2")
        .bind(recipient_new_balance)
        .bind(recipient_product.id)
        .execute(&mut *tx).await?;

    let sender_description = match note {
        Some(value) if !value.trim().is_empty() => format!("PayNow transfer to {}: {}", recipient_id, value.trim()),
        _ => format!("PayNow transfer to {}", recipient_id),
    };
    let recipient_description = format!("PayNow transfer from {}", sender_product.account_number);

    sqlx::query(
        "INSERT INTO transactions (product_id, transaction_type, amount_cents, balance_after_cents, description) VALUES ($1, 'paynow_transfer_out', $2, $3, $4)"
    )
    .bind(sender_product.id).bind(amount_cents).bind(sender_new_balance).bind(sender_description)
    .execute(&mut *tx).await?;

    sqlx::query(
        "INSERT INTO transactions (product_id, transaction_type, amount_cents, balance_after_cents, description) VALUES ($1, 'paynow_transfer_in', $2, $3, $4)"
    )
    .bind(recipient_product.id).bind(amount_cents).bind(recipient_new_balance).bind(recipient_description)
    .execute(&mut *tx).await?;

    tx.commit().await?;
    Ok((true, None))
}
