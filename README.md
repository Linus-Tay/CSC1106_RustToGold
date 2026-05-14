# RustToGold — Phase 1 MVC Foundation

RustToGold is a server-side rendered academic banking simulation built with Rust, Actix Web, Askama, SQLx, PostgreSQL, Argon2 password hashing, and cookie-based sessions.

This version uses a folder-based Rust module layout with `mod.rs` inside each major MVC folder. This keeps the code easier to navigate for a team project because each layer owns its own folder and exposes its public API through that folder's `mod.rs` file.

## What Phase 1 includes

- Original RustToGold landing, login, signup, dashboard, profile, and error page styling.
- Customer signup and login.
- Session-based authentication.
- Customer-only route protection.
- Default bank account created with `$0.00`.
- Deposit workflow.
- Transaction history generated from successful deposits.
- Connected placeholder routes for transfer, loans, loan application, fixed deposits, and new fixed deposit.
- PostgreSQL schema and setup script.
- Askama SSR template integration.
- Restored static asset paths:
  - `/static/assets/css/style.css`
  - `/static/assets/images/rusttogold-logo.png`
  - `/static/assets/js/app.js`

## Architecture

```text
src/
├── main.rs
├── config.rs
├── routes.rs
├── controllers/
│   ├── mod.rs
│   ├── auth_controller.rs
│   ├── customer_controller.rs
│   ├── error_controller.rs
│   ├── public_controller.rs
│   └── session_guard.rs
├── services/
│   ├── mod.rs
│   ├── auth_service.rs
│   ├── account_service.rs
│   ├── profile_service.rs
│   └── support.rs
├── repositories/
│   ├── mod.rs
│   ├── user_repository.rs
│   ├── account_repository.rs
│   └── transaction_repository.rs
├── models/
│   ├── mod.rs
│   ├── user.rs
│   ├── account.rs
│   ├── transaction.rs
│   ├── money.rs
│   └── formatting.rs
├── forms/
│   ├── mod.rs
│   ├── auth_forms.rs
│   ├── account_forms.rs
│   └── profile_forms.rs
└── views/
    ├── mod.rs
    ├── renderer.rs
    └── templates.rs

templates/
├── index.html
├── auth/
│   ├── login.html
│   └── signup.html
├── customer/
│   ├── dashboard.html
│   ├── deposit.html
│   ├── transactions.html
│   ├── profile.html
│   └── placeholder.html
└── errors/
    ├── 403.html
    ├── 404.html
    └── error.html

static/assets/
├── css/style.css
├── images/rusttogold-logo.png
└── js/app.js
```

## Rust module layout

The root module declarations remain in `src/main.rs`:

```rust
mod config;
mod controllers;
mod forms;
mod models;
mod repositories;
mod routes;
mod services;
mod views;
```

Because the project now uses folder modules, Rust will load these files:

```text
src/controllers/mod.rs
src/forms/mod.rs
src/models/mod.rs
src/repositories/mod.rs
src/services/mod.rs
src/views/mod.rs
```

Do not keep both of these at the same time:

```text
src/controllers.rs
src/controllers/mod.rs
```

That creates a duplicate module source conflict. The same rule applies to `forms`, `models`, `repositories`, `services`, and `views`.

## MVC mapping

- Model: `src/models/`
  - `User`, `BankAccount`, `Transaction`, `Money`
  - Rust `impl` blocks are used for model behavior such as display formatting and account rules.
  - `AccountWorkflow` trait demonstrates abstraction.

- View: `templates/` + `src/views/`
  - Askama templates render server-side HTML.
  - `src/views/templates.rs` defines the Askama template structs.
  - `src/views/renderer.rs` centralizes HTML rendering.

- Controller: `src/controllers/`
  - Receives requests, checks session state, calls services, and renders views.

- Service: `src/services/`
  - Contains business rules such as signup validation, login validation, deposit validation, and profile update validation.

- Repository: `src/repositories/`
  - Contains SQLx queries only.

- Forms: `src/forms/`
  - Contains request form structs for login, signup, deposit, and profile updates.

## OOP explanation for presentation

Rust is not class-based like Java, but this project still demonstrates OOP concepts:

- Encapsulation: structs group data, and `impl` blocks group behavior with the data.
- Abstraction: traits such as `AccountWorkflow` define account behavior without exposing implementation details.
- Separation of concerns: controllers do not directly talk to SQL; they call services and repositories.
- Maintainability: each module has one responsibility.
- Team collaboration: different members can work on separate controllers, services, repositories, models, or templates without constantly editing the same file.

## Setup instructions

### 1. Install requirements

You need:

- Rust toolchain
- PostgreSQL
- `psql` available in your terminal

On Windows, if `psql` is not recognized, either add PostgreSQL's `bin` folder to PATH or run it directly:

```powershell
& "C:\Program Files\PostgreSQL\18\bin\psql.exe" -U postgres
```

### 2. Create the database

```bash
psql -U postgres
```

Inside PostgreSQL:

```sql
CREATE DATABASE rusttogold;
\q
```

### 3. Create `.env`

Copy `.env.example` to `.env`.

Windows PowerShell:

```powershell
Copy-Item .env.example .env
```

macOS/Linux:

```bash
cp .env.example .env
```

Then edit `.env` and replace `YOUR_PASSWORD` with your PostgreSQL password.

Example:

```env
DATABASE_URL=postgres://postgres:yourActualPassword@localhost:5432/rusttogold
SESSION_SECRET=0123456789012345678901234567891201234567890123456789012345678912
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
```

### 4. Run the migration

From the project root:

```bash
psql -U postgres -d rusttogold -f migrations/001_init.sql
```

If Windows still cannot find `psql`, use the full path:

```powershell
& "C:\Program Files\PostgreSQL\18\bin\psql.exe" -U postgres -d rusttogold -f migrations/001_init.sql
```

### 5. Run the app

```bash
cargo run
```

Open:

```text
http://127.0.0.1:3000
```

## Demo flow

1. Open `/` and show the landing page.
2. Create a new customer account.
3. Dashboard starts at `$0.00`.
4. Click Deposit Money.
5. Deposit `$50.00`.
6. Open Transaction History.
7. Use the navbar to confirm all customer links resolve.
8. Logout and login again.

## Connected routes

```text
GET  /                              Landing page
GET  /signup                        Signup page
POST /signup                        Create customer + default $0 account
GET  /login                         Login page
POST /login                         Authenticate user
GET  /logout                        Logout
GET  /403                           Access denied page
GET  /customer/dashboard            Customer dashboard
GET  /customer/deposit              Deposit page
POST /customer/deposit              Create deposit transaction and update balance
GET  /customer/transfer             Phase 1 connected placeholder
GET  /customer/transactions         Transaction history
GET  /customer/loans                Phase 1 connected placeholder
GET  /customer/loans/apply          Phase 1 connected placeholder
GET  /customer/fixed-deposits       Phase 1 connected placeholder
GET  /customer/fixed-deposits/new   Phase 1 connected placeholder
GET  /customer/profile              Profile page
POST /customer/profile              Update profile handler
```

## Database tables

- `users`
- `bank_accounts`
- `transactions`

Phase 2 can add real transfer processing, loan application persistence, fixed deposit persistence, OTP simulation, concurrency-safe transfers, audit logging, staff dashboard, admin dashboard, and fraud rules.
