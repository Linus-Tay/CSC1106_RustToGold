# RustToGold - CSC1106 Banking System

RustToGold is a server-side rendered academic banking simulation built with Rust, Actix Web, Askama, SQLx, PostgreSQL, Argon2 password hashing, and cookie-based sessions.

This project uses a simple MVC-style folder structure:

- **Controllers** receive HTTP requests and render pages.
- **Services** contain validation and banking rules.
- **Repositories** contain SQLx database queries.
- **Models** contain Rust structs and small `impl` methods for display helpers.
- **Forms** contain `web::Form` request data structs.
- **Templates** are Askama SSR HTML pages.

## Fixed Deposit System

The Fixed Deposit module supports:

1. Customer creation of fixed deposits using active plans.
2. PostgreSQL storage of FD records.
3. Simple interest calculation.
4. Customer FD dashboard with balance, status, interest, maturity date, and payout.
5. Maturity checking whenever the dashboard or withdrawal route is opened.
6. Matured payout: principal plus interest.
7. Early withdrawal: principal returned and interest forfeited.
8. Status tracking: `active`, `matured`, `withdrawn`, `paid_out`, and `cancelled`.
9. Transaction records for FD opening, early withdrawal, and payout.
10. Admin FD record viewing and FD plan creation, updating, activation, and deactivation.

### Interest formula

The system uses simple interest:

```text
interest = principal x annual interest rate x duration in months / 12
```

Money is stored as integer cents. Interest rates are stored as basis points, so `320` means `3.20%`.

## Fixed Deposit routes

### Customer

```text
GET  /customer/fixed-deposits
GET  /customer/fixed-deposits/new
POST /customer/fixed-deposits
POST /customer/fixed-deposits/{id}/withdraw
```

### Admin

```text
GET  /admin/fixed-deposits
GET  /admin/fixed-deposit-plans
POST /admin/fixed-deposit-plans
POST /admin/fixed-deposit-plans/{id}
```

## Database setup

### 1. Create the database

```sql
CREATE DATABASE rusttogold;
```

### 2. Create `.env`

Copy `.env.example` to `.env` and update the PostgreSQL password.

PowerShell:

```powershell
Copy-Item .env.example .env
```

Example `.env`:

```env
DATABASE_URL=postgres://postgres:your_password@localhost:5432/rusttogold
SESSION_SECRET=0123456789012345678901234567891201234567890123456789012345678912
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
```

### 3. Run migrations

These migration files are non-destructive. They do not contain `DROP TABLE` commands.

```powershell
psql -U postgres -d rusttogold -f migrations/001_init.sql
psql -U postgres -d rusttogold -f migrations/002_fixed_deposits.sql
```

### 4. Check and run

```powershell
cargo check
cargo run
```

Open the website at:

```text
http://127.0.0.1:3000
```

## Admin test account

Customer accounts are created from the signup page. To test the admin pages, use pgAdmin Query Tool or `psql` to change one test user's role:

```sql
UPDATE users
SET role = 'admin'
WHERE email = 'your_test_email@example.com';
```

Logout and log in again. Admin users are redirected to `/admin/fixed-deposits`.

## Fixed Deposit test checklist

1. Sign up and deposit money into the customer account.
2. Create an FD with a valid active plan and enough balance.
3. Try an amount below the plan minimum; the page should show an error.
4. Try an amount higher than the available balance; the page should show an error.
5. Withdraw an active FD; principal returns and the interest becomes the penalty.
6. Change a test FD maturity date to today or an earlier date, open the dashboard, then claim payout. Principal plus interest should return.
7. Try the same withdrawal/payout again; it should fail.
8. Log in as admin and create/update a plan.
9. Set a plan to inactive and confirm it disappears from the customer FD creation page.
10. As a customer, visit an `/admin/...` URL and confirm the app redirects to `/403`.

## Frontend handoff

The templates are functional. Frontend work can focus on visual polish: responsive table layout, clearer status badges, withdrawal confirmation wording, and consistent success/error messages. Banking calculations, maturity checks, status changes, and payout decisions are kept in the Rust backend.
