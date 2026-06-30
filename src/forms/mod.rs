pub mod account_forms;
pub mod auth_forms;
pub mod fixed_deposit_forms;
pub mod home_loan_forms;
pub mod loan_forms;
pub mod onboard_forms;
pub mod profile_forms;

pub use self::account_forms::{CreateBankAccountForm, DepositForm};
pub use self::auth_forms::{
    AccountCreationForm, LoginForm, SignupAccountForm, SignupContactForm, SignupDeclarationForm, SignupDraft,
    SignupEmploymentForm, SignupForm, SignupPersonalForm, SignupSecurityForm,
};
pub use self::fixed_deposit_forms::{
    CreateFixedDepositForm, FixedDepositMessageQuery, FixedDepositPlanForm,
};
pub use self::home_loan_forms::{HomeLoanApplicationForm, HomeLoanPaymentForm};
pub use self::loan_forms::{LoanApplicationForm, LoanPaymentForm};
pub use self::onboard_forms::{OnboardingForm, Step1Form, Step2Form, Step3Form, Step4Form};
pub use self::profile_forms::ProfileForm;
