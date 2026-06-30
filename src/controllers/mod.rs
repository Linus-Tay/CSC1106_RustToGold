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
    admin_dashboard, admin_personal_loans_page, admin_signups_page, approve_customer_application,
    approve_personal_loan, reject_customer_application, reject_personal_loan,
};
pub use self::auth_controller::{
    admin_login, admin_login_page, admin_logout, login, login_page, logout, post_signup_account, post_signup_contact, post_signup_employment,
    post_signup_personal, post_signup_security, post_signup_submit, show_signup_account,
    show_signup_contact, show_signup_employment, show_signup_personal, show_signup_review,
    show_signup_security, signup, signup_page,
};
pub use self::customer_controller::{
    approve_product, dashboard, deposit, deposit_page, fixed_deposit_activity, loan_activity,
    profile_page, transactions, transfer, transfer_page, update_profile,
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
pub use self::onboard_controller::{onboarding, step1_post, submit};
pub use self::public_controller::{about_page, banking_page, contact_page, faq_page, home, security_page};
