pub mod account_service;
pub mod auth_service;
pub mod fixed_deposit_service;
pub mod profile_service;
pub mod staff_service;
pub mod support;
pub mod audit_log_service;

pub use self::account_service::{deposit, list_transactions, load_customer_dashboard, load_admin_transactions, load_admin_accounts, update_account_status, AdminTransactionPage};
pub use self::fixed_deposit_service::{create_fixed_deposit, create_fixed_deposit_plan, list_all_fixed_deposit_plans, list_all_fixed_deposits, load_create_fixed_deposit_page, load_fixed_deposit_dashboard, update_fixed_deposit_plan, withdraw_fixed_deposit, FixedDepositDashboardData};
pub use self::auth_service::{authenticate_user, register_customer};
pub use self::profile_service::update_customer_profile;
pub use self::staff_service::{list_all_staff, create_staff, find_staff_by_id, update_staff, delete_staff};
pub use self::audit_log_service::{AuditContext, load_audit_log_page, list_for_entity, record, record_simple};