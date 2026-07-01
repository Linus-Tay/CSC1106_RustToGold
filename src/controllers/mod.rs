// Controller layer: maps routes to services and chooses which template/redirect to return.
pub mod admin_controller;
pub mod auth_controller;
pub mod customer_controller;
pub mod error_controller;
pub mod fixed_deposit_controller;
pub mod home_loan_controller;
pub mod loan_controller;
pub mod onboard_controller;
pub mod public_controller;
pub mod session_guard;

pub use self::admin_controller::{
    activate_customer_product, activate_customer_user, admin_audit_log_page,
    admin_customer_accounts_page, admin_dashboard, admin_high_value_monitoring_page,
    admin_personal_loans_page, admin_signups_page, admin_staff_page, approve_customer_application, approve_personal_loan, close_customer_product,
    create_staff_user, delete_staff_user, freeze_customer_product, reject_customer_application,
    reject_personal_loan, suspend_customer_user, update_high_value_alert_status, update_staff_user,
};
pub use self::auth_controller::{admin_login, admin_login_page, admin_logout, login, login_page, logout};
pub use self::customer_controller::{
    activate_card, cancel_giro_arrangement, cards_page, create_bank_account, create_card,
    create_giro_arrangement, dashboard, deposit, deposit_page, download_statement_pdf,
    fixed_deposit_activity, freeze_card, giro_page, loan_activity, paynow_page,
    profile_page, register_paynow, statements_page, transaction_controls_page,
    transactions, transfer, transfer_page, transfer_paynow, update_money_lock,
    update_profile, update_transaction_limit,
};
pub use self::error_controller::{forbidden, not_found};
pub use self::fixed_deposit_controller::{
    admin_fixed_deposit_plans_page, admin_fixed_deposits_page, create_fixed_deposit,
    create_fixed_deposit_plan, fixed_deposit_new_page, fixed_deposits_page,
    update_fixed_deposit_plan, withdraw_fixed_deposit,
};
pub use self::home_loan_controller::{
    admin_home_loans_page, approve_home_loan, create_home_loan_application, home_loan_apply_page,
    home_loans_page, pay_home_loan, reject_home_loan,
};
pub use self::loan_controller::{create_personal_loan, loan_apply_page, loans_page, pay_loan};
pub use self::onboard_controller::{account_creation, account_creation_init, account_creation_submit, legacy_signup_path_redirect, legacy_signup_redirect, onboarding, onboarding_entry_redirect, step1_post, step2_post, step3_post, step4_post, submit};
pub use self::public_controller::{about_page, banking_page, contact_page, faq_page, home, security_page};
