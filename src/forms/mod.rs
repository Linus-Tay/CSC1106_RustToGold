// Form layer: request payload structs received from HTML forms.

pub mod account_forms;
pub mod auth_forms;
pub mod card_forms;
pub mod fixed_deposit_forms;
pub mod home_loan_forms;
pub mod giro_forms;
pub mod loan_forms;
pub mod monitoring_forms;
pub mod onboard_forms;
pub mod paynow_forms;
pub mod profile_forms;
pub mod statement_forms;
pub mod transaction_control_forms;
pub mod atm_forms;

pub use self::account_forms::DepositForm;
pub use self::auth_forms::{AccountCreationForm, LoginForm};
pub use self::card_forms::CardApplicationForm;
pub use self::fixed_deposit_forms::{
    CreateFixedDepositForm, FixedDepositMessageQuery, FixedDepositPlanForm,
};
pub use self::giro_forms::GiroArrangementForm;
pub use self::home_loan_forms::{HomeLoanApplicationForm, HomeLoanPaymentForm};
pub use self::loan_forms::{LoanApplicationForm, LoanPaymentForm};
pub use self::monitoring_forms::MonitoringStatusForm;
pub use self::onboard_forms::{OnboardingForm, Step1Form, Step2Form};
pub use self::paynow_forms::{PayNowRegisterForm, PayNowTransferForm};
pub use self::profile_forms::ProfileForm;
pub use self::statement_forms::StatementRequest;
pub use self::transaction_control_forms::{MoneyLockForm, TransactionLimitForm};
