pub mod account_service;
pub mod admin_service;
pub mod auth_service;
pub mod customer_service;
pub mod email_service;
pub mod fixed_deposit_service;
pub mod home_loan_service;
pub mod loan_service;
pub mod onboard_service;
pub mod product_service;
pub mod profile_service;
pub mod support;

pub use self::account_service::{
    list_fixed_deposit_activity, list_loan_activity, list_transactions, load_customer_dashboard,
};
pub use self::admin_service::{
    approve_customer_application, approve_personal_loan, list_admin_customer_applications,
    list_admin_personal_loans, load_admin_dashboard, reject_customer_application,
    reject_personal_loan,
};
pub use self::auth_service::{authenticate_user, register_customer, register_user, submit_customer_application};
pub use self::customer_service::{create_customer, create_customer_with_product, approve_customer_with_product, validate_account_creation_link, get_customer_by_account_creation_link, invalidate_account_creation_link};
pub use self::email_service::send_template_email;
pub use self::fixed_deposit_service::{
    create_fixed_deposit, create_plan, list_admin_fixed_deposits, list_admin_plans,
    load_fixed_deposit_create_page, load_fixed_deposit_dashboard, update_plan,
    withdraw_fixed_deposit, FixedDepositDashboard,
};
pub use self::home_loan_service::{
    approve_home_loan, list_admin_home_loans, load_home_loan_dashboard, pay_home_loan,
    reject_home_loan, submit_home_loan_application, HomeLoanDashboard,
};
pub use self::loan_service::{apply_personal_loan, load_loan_dashboard, pay_personal_loan, LoanDashboard};
pub use self::onboard_service::{get_path_template, get_product_details};
pub use self::product_service::{approve_product, create_product, deposit, generate_account_number, transfer};
pub use self::profile_service::update_customer_profile;
