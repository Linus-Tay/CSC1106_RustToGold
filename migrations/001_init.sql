-- RustToGold 001_init.sql
-- Updated to match the current Rust models/repositories and the routed account-creation flow.
-- WARNING: This is a reset script. It drops existing local development tables.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

DROP TABLE IF EXISTS transactions CASCADE;
DROP TABLE IF EXISTS registered_paynow CASCADE;
DROP TABLE IF EXISTS account_creation_links CASCADE;
DROP TABLE IF EXISTS bank_accounts CASCADE;
DROP TABLE IF EXISTS customer_products CASCADE;
DROP TABLE IF EXISTS users CASCADE;
DROP TABLE IF EXISTS customers CASCADE;

CREATE TABLE customers (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    full_name            TEXT NOT NULL,
    nric                 TEXT UNIQUE NOT NULL,
    date_of_birth        DATE NOT NULL,
    gender               TEXT NOT NULL,
    nationality          TEXT NOT NULL,
    residency            TEXT NOT NULL,
    race                 TEXT NULL,
    email                TEXT UNIQUE NOT NULL,
    phone_number         TEXT NOT NULL,
    residential_address  TEXT NOT NULL,
    mailing_address      TEXT NULL,
    preferred_contact    TEXT NULL,
    employment_status    TEXT NOT NULL,
    occupation           TEXT NULL,
    employer_name        TEXT NULL,
    industry             TEXT NULL,
    monthly_income_range TEXT NULL,
    kyc_status           TEXT NOT NULL DEFAULT 'pending',
    created_at           TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at           TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT customers_kyc_status_check
        CHECK (kyc_status IN ('pending', 'approved', 'rejected'))
);

CREATE TABLE users (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id    UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    email          TEXT NOT NULL UNIQUE,
    password_hash  TEXT NOT NULL,
    role           TEXT NOT NULL DEFAULT 'customer',
    status         TEXT NOT NULL DEFAULT 'active',
    last_login_at  TIMESTAMP NULL,
    created_at     TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT users_role_check
        CHECK (role IN ('customer', 'staff', 'admin')),
    CONSTRAINT users_status_check
        CHECK (status IN ('active', 'suspended', 'closed'))
);

-- CREATE TABLE bank_accounts (
--     id             BIGSERIAL PRIMARY KEY,
--     user_id        BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
--     account_number TEXT NOT NULL UNIQUE,
--     account_type   TEXT NOT NULL DEFAULT 'everyday_savings',
--     balance_cents  BIGINT NOT NULL DEFAULT 0,
--     status         TEXT NOT NULL DEFAULT 'active',
--     created_at     TIMESTAMP NOT NULL DEFAULT NOW(),
--     updated_at     TIMESTAMP NOT NULL DEFAULT NOW(),

--     CONSTRAINT bank_accounts_type_check
--         CHECK (account_type IN ('everyday_savings', 'high_yield_savings', 'savings', 'current')),
--     CONSTRAINT bank_accounts_status_check
--         CHECK (status IN ('active', 'frozen', 'closed')),
--     CONSTRAINT bank_accounts_balance_non_negative
--         CHECK (balance_cents >= 0)
-- );

CREATE TABLE customer_products (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id    UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    product_id     TEXT NOT NULL,
    product_type   TEXT NOT NULL,
    account_number TEXT NOT NULL UNIQUE,
    status         TEXT NOT NULL DEFAULT 'inactive',
    balance_cents  BIGINT NOT NULL DEFAULT 0,
    created_at     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT customer_products_unique_customer_product
        UNIQUE (customer_id, product_id),
    CONSTRAINT customer_products_status_check
        CHECK (status IN ('active', 'inactive', 'closed')),
    CONSTRAINT customer_products_balance_non_negative
        CHECK (balance_cents >= 0),
    CONSTRAINT customer_products_type_check
        CHECK (product_type IN ('savings', 'spending', 'fixed_deposit', 'investment'))
);

CREATE TABLE transactions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id          UUID NULL REFERENCES customer_products(id) ON DELETE SET NULL,
    customer_id         UUID NULL REFERENCES customers(id) ON DELETE SET NULL,
    transaction_type    TEXT NOT NULL,
    amount_cents        BIGINT NOT NULL,
    balance_after_cents BIGINT NOT NULL,
    description         TEXT NULL,
    created_at          TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT transactions_type_check
        CHECK (transaction_type IN ('deposit', 'withdrawal', 'transfer_in', 'transfer_out')),
    CONSTRAINT transactions_amount_positive
        CHECK (amount_cents > 0),
    CONSTRAINT transactions_balance_after_non_negative
        CHECK (balance_after_cents >= 0)
);

CREATE TABLE account_creation_links (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    status      TEXT NOT NULL DEFAULT 'pending',
    expires_at  TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT account_creation_links_status_check
        CHECK (status IN ('pending', 'expired', 'used'))
);

CREATE TABLE registered_paynow (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id       UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    paynow_type       TEXT NOT NULL,
    paynow_id         TEXT NOT NULL,
    linked_account_id UUID NOT NULL REFERENCES customer_products(id) ON DELETE CASCADE,
    registered_at     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status            TEXT NOT NULL DEFAULT 'active',

    CONSTRAINT registered_paynow_type_check
        CHECK (paynow_type IN ('phone_number', 'nric')),
    CONSTRAINT registered_paynow_status_check
        CHECK (status IN ('active', 'inactive'))
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_customer_id ON users(customer_id);
--CREATE INDEX idx_bank_accounts_user_id ON bank_accounts(user_id);
CREATE INDEX idx_customer_products_customer_id ON customer_products(customer_id);
CREATE INDEX idx_customer_products_account_number ON customer_products(account_number);
--CREATE INDEX idx_transactions_user_id_created_at ON transactions(user_id, created_at DESC);
CREATE INDEX idx_transactions_customer_id_created_at ON transactions(customer_id, created_at DESC);
CREATE INDEX idx_transactions_product_id_created_at ON transactions(product_id, created_at DESC);
