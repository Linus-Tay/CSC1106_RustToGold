pub mod account_service;
pub mod auth_service;
pub mod fixed_deposit_service;
pub mod loan_service;
pub mod home_loan_service;
pub mod profile_service;
pub mod support;

pub use self::account_service::{deposit, list_transactions, load_customer_dashboard};
pub use self::auth_service::{authenticate_user, register_customer};
pub use self::fixed_deposit_service::{
    create_fixed_deposit, create_fixed_deposit_plan, list_all_fixed_deposit_plans,
    list_all_fixed_deposit_records, load_create_fixed_deposit_page,
    load_fixed_deposit_dashboard, update_fixed_deposit_plan, withdraw_fixed_deposit,
};
pub use self::profile_service::update_customer_profile;
pub use self::loan_service::{apply_personal_loan, load_loan_dashboard, pay_loan};
pub use self::home_loan_service::{apply_home_loan,approve_home_loan,reject_home_loan,
    pay_home_loan,load_home_loan_dashboard,list_all_home_loans_for_admin,
};