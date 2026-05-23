pub mod account_service;
pub mod auth_service;
pub mod fixed_deposit_service;
pub mod profile_service;
pub mod support;

pub use self::account_service::{deposit, list_transactions, load_customer_dashboard};
pub use self::fixed_deposit_service::{create_fixed_deposit, create_fixed_deposit_plan, list_all_fixed_deposit_plans, list_all_fixed_deposits, load_create_fixed_deposit_page, load_fixed_deposit_dashboard, update_fixed_deposit_plan, withdraw_fixed_deposit, FixedDepositDashboardData};
pub use self::auth_service::{authenticate_user, register_customer};
pub use self::profile_service::update_customer_profile;
