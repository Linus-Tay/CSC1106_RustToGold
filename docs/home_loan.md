# Fixed Deposit Test Checklist

Run the app after both SQL migration files have been applied.

## Customer tests

1. **Valid FD creation**
   - Deposit money into a customer account.
   - Open `/customer/fixed-deposits/new`.
   - Select an active plan and enter an amount that meets the minimum.
   - Expected: FD becomes `active`, account balance decreases, and a `fixed_deposit_opening` transaction appears.

2. **Below-minimum rejection**
   - Enter an amount lower than the selected plan's minimum.
   - Expected: form shows the minimum amount error; account balance and FD records do not change.

3. **Insufficient-balance rejection**
   - Enter an amount higher than available balance.
   - Expected: form shows the insufficient balance error; no FD is created.

4. **Early withdrawal**
   - Create an active FD and press **Withdraw Early**.
   - Expected: status becomes `withdrawn`, principal returns, `penalty_cents` equals the expected interest, and a `fixed_deposit_early_withdrawal` transaction appears.

5. **Matured payout**
   - For a test FD, set `maturity_date` to today in pgAdmin Query Tool:

     ```sql
     UPDATE fixed_deposits
     SET maturity_date = CURRENT_DATE
     WHERE id = your_fd_id;
     ```

   - Refresh the FD dashboard, then press **Claim Payout**.
   - Expected: status becomes `paid_out`, principal plus interest returns, and a `fixed_deposit_payout` transaction appears.

6. **Double payout prevention**
   - Press the withdrawal/payout route again for a completed FD.
   - Expected: the app rejects it and no second transaction or balance increase occurs.

## Admin tests

7. **Plan creation and update**
   - Change a test customer to admin in pgAdmin:

     ```sql
     UPDATE users
     SET role = 'admin'
     WHERE email = 'your_test_email@example.com';
     ```

   - Logout, login again, then open `/admin/fixed-deposit-plans`.
   - Create a plan and update a plan.
   - Expected: changes appear in `fixed_deposit_plans`.

8. **Active plans only for customer**
   - Set a plan to `inactive` in admin page.
   - Login as a customer and open `/customer/fixed-deposits/new`.
   - Expected: inactive plan does not appear.

9. **Admin page protection**
   - As a customer, open `/admin/fixed-deposits`.
   - Expected: redirect to `/403`.
