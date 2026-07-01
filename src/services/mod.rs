// Service layer: business rules live here so controllers stay thin and repositories stay database-focused.
pub mod account_service;
pub mod admin_service;
pub mod auth_service;
pub mod card_service;
pub mod customer_service;
pub mod email_service;
pub mod fixed_deposit_service;
pub mod home_loan_service;
pub mod giro_service;
pub mod loan_service;
pub mod monitoring_service;
pub mod onboard_service;
pub mod paynow_service;
pub mod product_service;
pub mod profile_service;
pub mod statement_service;
pub mod support;
pub mod transaction_control_service;

pub use self::account_service::{
    list_fixed_deposit_activity, list_loan_activity, list_transactions, load_customer_dashboard,
};
pub use self::admin_service::{
    approve_customer_application, approve_personal_loan, create_staff_user,
    delete_staff_user, list_admin_customer_accounts, list_admin_customer_applications,
    list_admin_personal_loans, list_admin_staff, list_audit_logs,
    load_admin_dashboard, reject_customer_application, reject_personal_loan, set_customer_product_status,
    set_customer_user_status, update_staff_user,
};
pub use self::auth_service::{authenticate_user, register_user, authenticate_device, add_trusted_device, generate_and_send_2fa};
pub use self::card_service::{create_card, load_card_dashboard, set_card_status};
pub use self::customer_service::{create_customer_with_product, approve_customer_with_product, validate_account_creation_link, get_customer_by_account_creation_link, invalidate_account_creation_link};
pub use self::email_service::{send_html_email, send_template_email};
pub use self::fixed_deposit_service::{
    create_fixed_deposit, create_plan, list_admin_fixed_deposits, list_admin_plans,
    load_fixed_deposit_create_page, load_fixed_deposit_dashboard, update_plan,
    withdraw_fixed_deposit,
};
pub use self::giro_service::{cancel_giro_arrangement, create_giro_arrangement, load_giro_dashboard};
pub use self::home_loan_service::{
    approve_home_loan, list_admin_home_loans, load_home_loan_dashboard, pay_home_loan,
    reject_home_loan, submit_home_loan_application,
};
pub use self::loan_service::{apply_personal_loan, load_loan_dashboard, pay_personal_loan};
pub use self::monitoring_service::{load_high_value_monitoring_dashboard, update_high_value_alert_status};
pub use self::paynow_service::{load_paynow_dashboard, register_paynow, transfer_paynow, PayNowDashboard};
pub use self::product_service::{create_bank_account, deposit, generate_account_number, list_active_customer_products, transfer};
pub use self::profile_service::update_customer_profile;
pub use self::statement_service::{build_bank_statement, load_statement_page, render_statement_pdf, statement_pdf_filename};
pub use self::transaction_control_service::{load_transaction_controls_page, update_daily_transaction_limit, update_money_lock};
