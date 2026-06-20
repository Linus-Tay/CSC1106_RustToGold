# Fixed Deposit Frontend Handoff

The Fixed Deposit backend is completed. The frontend should keep the existing SSR pages and improve the UI only. Do not move banking calculations into JavaScript.

## Backend routes

### Customer routes

| Method | Route | Purpose |
|---|---|---|
| GET | `/customer/fixed-deposits` | Customer FD dashboard |
| GET | `/customer/fixed-deposits/new` | Create FD form |
| POST | `/customer/fixed-deposits` | Submit a new FD |
| POST | `/customer/fixed-deposits/{id}/withdraw` | Early withdrawal or matured payout |

### Admin routes

| Method | Route | Purpose |
|---|---|---|
| GET | `/admin/fixed-deposits` | View all customer FDs |
| GET | `/admin/fixed-deposit-plans` | View and manage FD plans |
| POST | `/admin/fixed-deposit-plans` | Create FD plan |
| POST | `/admin/fixed-deposit-plans/{id}` | Update / activate / deactivate plan |

## Pages to polish

### 1. Customer FD Dashboard

File: `templates/customer/fixed_deposits.html`

Keep these details visible:

- Account number and available balance
- Active and matured FD counts
- Principal currently locked
- Expected interest
- FD table with principal, annual rate, interest, maturity date, payout, status, and action

Status badges:

- `active`: green
- `matured`: blue
- `withdrawn`: grey
- `paid_out`: green
- `cancelled`: red

Actions:

- `active`: show **Withdraw Early**
- `matured`: show **Claim Payout**
- `withdrawn`, `paid_out`, `cancelled`: show **No action**

For active records, show clear wording that early withdrawal returns principal only and forfeits interest.

### 2. Create Fixed Deposit Form

File: `templates/customer/fixed_deposit_new.html`

Use normal HTML form controls with native validation:

- Plan dropdown: `name="plan_id"`, required
- Amount input: `name="amount"`, `type="number"`, `min="0.01"`, `step="0.01"`, required
- Submit button

The plan dropdown is already filled by the backend with active plans only. Keep this explanatory message on the page:

> Interest is calculated by the backend. Money is locked until maturity. Early withdrawal returns principal only.

### 3. Customer messages

Use the existing alert areas to show backend results:

- Successful FD creation
- Successful matured payout
- Successful early withdrawal
- Invalid amount, insufficient balance, invalid plan, and plan minimum errors

Do not calculate or decide any of these messages in JavaScript. The Rust controller/service decides the business result.

### 4. Admin FD Records

File: `templates/admin/fixed_deposits.html`

Keep the table readable on desktop and scrollable on small screens. It should show:

- FD ID
- Customer name and email
- Account number
- Plan name
- Principal
- Interest rate and interest amount
- Payout
- Maturity date
- Status badge

### 5. Admin FD Plan Management

File: `templates/admin/fixed_deposit_plans.html`

The create form fields must remain exactly:

- `name`
- `duration_months`
- `interest_rate`
- `minimum_amount`
- `status`

For each existing plan, keep the update form route:

```text
POST /admin/fixed-deposit-plans/{id}
```

`status` must allow:

- `active`
- `inactive`

An inactive plan must stay visible to admin but must not show on the customer create FD page.

## Important frontend rules

Do not calculate these in HTML or JavaScript:

- Interest
- Maturity date
- Payout amount
- Penalty amount
- FD status
- Whether a customer can receive a payout

The backend already calculates and validates all banking rules. The frontend only renders values and sends form submissions.
