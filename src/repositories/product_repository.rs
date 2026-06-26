use crate::models::{BankAccount, Product, Transaction, product};
use sqlx::{PgPool, Postgres, Transaction as DbTransaction};
use uuid::Uuid;

pub async fn get_product_by_user_id_and_product_id(db: &PgPool, customer_id: &Uuid, product_id: &str) -> Result<Option<Product>, sqlx::Error> {
    sqlx::query_as::<_, Product> (
        r#"
        SELECT id, customer_id, account_number, product_id, balance_cents, status, created_at, updated_at FROM customer_products WHERE product_id = $1 AND customer_id = $2
        "#
    )
    .bind(product_id)
    .bind(customer_id)
    .fetch_optional(db)
    .await
}

pub async fn get_product_by_account_number(db: &PgPool, account_number: &str) -> Result<Option<Product>, sqlx::Error> {
    sqlx::query_as::<_, Product> (
        r#"
        SELECT id, customer_id, account_number, product_id, product_type, balance_cents, status, created_at, updated_at FROM customer_products WHERE account_number = $1
        "#
    )
    .bind(account_number)
    .fetch_optional(db)
    .await
}

pub async fn insert_product(db: &PgPool, customer_id: &Uuid, product_id: &str, product_type: &str, account_number: &str) -> Result<Product, sqlx::Error> {
    sqlx::query_as::<_, Product>(
        r#"
        INSERT INTO customer_products (customer_id, product_id, product_type, account_number, balance_cents)
        VALUES ($1, $2, $3, $4, 0)
        RETURNING id, customer_id, account_number, product_id, product_type, balance_cents, status, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .bind(product_id)
    .bind(product_type)
    .bind(account_number)
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

pub async fn deposit_into_product(db: &PgPool, customer_id: &Uuid, account_number: &str, amount_cents: i64, description: Option<&str>,) -> Result<(Product, Transaction), sqlx::Error> {
    println!("this ran??");
    let mut tx = db.begin().await?;
    let product = lock_product(&mut tx, customer_id, account_number).await?;
    
    println!("old balance: {}", product.balance_cents);
    println!("amount to add: {}", amount_cents);

    let new_balance = product.balance_cents + amount_cents;

    println!("{}", "here");
    println!("{}", product.account_number);

    let updated_product = sqlx::query_as::<_, Product>(
        r#"
        UPDATE customer_products
        SET balance_cents = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, customer_id, account_number, product_id, product_type, balance_cents, status, created_at, updated_at
        "#,
    )
    .bind(new_balance)
    .bind(product.id)
    .fetch_one(&mut *tx)
    .await?;

    println!("okkk??");

    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (product_id, customer_id, transaction_type, amount_cents, balance_after_cents, description)
        VALUES ($1, $2, 'deposit', $3, $4, $5)
        RETURNING id, product_id, customer_id, transaction_type, amount_cents, balance_after_cents, description, created_at
        "#,
    )
    .bind(product.id)
    .bind(customer_id)
    .bind(amount_cents)
    .bind(new_balance)
    .bind(description)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    println!("{}", "this is a test");
    println!("{}", updated_product.balance_cents);

    Ok((updated_product, transaction))
}

pub async fn transfer(db: &PgPool, sender_account_number: &str, sender_customer_id: &Uuid, recipient_customer_id: &Uuid, recipient_account_number: &str, amount_cents: i64, note: Option<&str>) -> Result<(bool, Option<String>), sqlx::Error> {
    if sender_account_number == recipient_account_number {
        return Ok((false, Some(String::from("You cannot transfer to the same bank account"))));
    }

    println!("sender: {}, {}", sender_customer_id, sender_account_number);
    println!("recipient: {} {}", recipient_customer_id, recipient_account_number);


    let mut tx = db.begin().await?;

    let (sender_product, recipient_product) = if sender_account_number < recipient_account_number {
        println!("we try to lock sender first");        
        let first = lock_product(&mut tx, sender_customer_id, sender_account_number).await?;
        println!("sender can be locked?");
        let second = lock_product(&mut tx, recipient_customer_id, recipient_account_number).await?;
        println!("recipeient can be locked?");
        
        (first, second)
        
    } else { 
        println!("we try to lock recipient first");
        let first = lock_product(&mut tx, recipient_customer_id, recipient_account_number).await?;
        let second = lock_product(&mut tx, sender_customer_id, sender_account_number).await?;
        
        (second, first)
    };

    println!("can i fking lock?");

    if sender_product.balance_cents < amount_cents {
        return Ok((false, Some(String::from("Insufficient funds"))))
    }

    let sender_new_balance = sender_product.balance_cents - amount_cents;
    let recipient_new_balance = recipient_product.balance_cents + amount_cents;

    sqlx::query_as::<_, Product>(
        r#"
        UPDATE customer_products
        SET balance_cents = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, customer_id, account_number, product_id, product_type, balance_cents, status, created_at, updated_at
        "#,
    )
    .bind(sender_new_balance)
    .bind(sender_product.id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query_as::<_, Product>(
        r#"
        UPDATE customer_products
        SET balance_cents = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, customer_id, account_number, product_id, product_type, balance_cents, status, created_at, updated_at
        "#,
    )
    .bind(recipient_new_balance)
    .bind(recipient_product.id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (product_id, customer_id, transaction_type, amount_cents, balance_after_cents, description)
        VALUES ($1, $2, 'transfer_out', $3, $4, $5)
        RETURNING id, product_id, customer_id, transaction_type, amount_cents, balance_after_cents, description, created_at
        "#,
    )
    .bind(sender_product.id)
    .bind(sender_customer_id)
    .bind(amount_cents)
    .bind(sender_new_balance)
    .bind(note)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (product_id, customer_id, transaction_type, amount_cents, balance_after_cents, description)
        VALUES ($1, $2, 'transfer_in', $3, $4, $5)
        RETURNING id, product_id, customer_id, transaction_type, amount_cents, balance_after_cents, description, created_at
        "#,
    )
    .bind(recipient_product.id)
    .bind(recipient_customer_id)
    .bind(amount_cents)
    .bind(recipient_new_balance)
    .bind(note)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((true, None))
}

async fn lock_product(tx: &mut DbTransaction<'_, Postgres>, customer_id: &Uuid, account_number: &str) -> Result<Product, sqlx::Error> {
    println!("shit here");
    println!("{} {}", customer_id, account_number);
    sqlx::query_as::<_, Product>(
        r#"
        SELECT id, customer_id, account_number, product_id, product_type, balance_cents, status, created_at, updated_at
        FROM customer_products
        WHERE customer_id = $1 AND account_number = $2 AND status = 'active'
        ORDER BY id ASC
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(customer_id)
    .bind(account_number)
    .fetch_one(&mut **tx)
    .await
}
