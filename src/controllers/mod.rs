pub mod account_controller;
pub mod auth_controller;
pub mod customer_controller;
pub mod error_controller;
pub mod fixed_deposit_controller;
pub mod public_controller;
pub mod session_guard;
pub mod loan_controller;
pub mod home_loan_controller;

pub use self::loan_controller::*;
pub use self::home_loan_controller::{create_home_loan_application, home_loan_apply_page, pay_home_loan};
pub use self::account_controller::{create, create_page, display_product};
pub use self::auth_controller::{login, login_page, logout, signup, signup_page};
pub use self::customer_controller::{
    dashboard, deposit, deposit_page, loan_apply_page, loans_page, profile_page, transactions,
    transfer_page, update_profile,
};
pub use self::fixed_deposit_controller::{
    admin_fixed_deposit_plans_page, admin_fixed_deposits_page, create_fixed_deposit,
    create_fixed_deposit_plan, fixed_deposit_new_page, fixed_deposits_page,
    update_fixed_deposit_plan, withdraw_fixed_deposit,
};
pub use self::error_controller::{forbidden, not_found};
pub use self::public_controller::home;
