pub mod account_forms;
pub mod auth_forms;
pub mod fixed_deposit_forms;
pub mod profile_forms;
pub mod loan_forms;
pub mod home_loan_forms;

pub use self::account_forms::DepositForm;
pub use self::auth_forms::{LoginForm, SignupForm};
pub use self::fixed_deposit_forms::{CreateFixedDepositForm, FixedDepositPlanForm};
pub use self::profile_forms::ProfileForm;
pub use self::loan_forms::{LoanApplicationForm, LoanPaymentForm};
pub use self::home_loan_forms::*;
