pub mod renderer;
pub mod templates;

pub use self::renderer::render;
pub use self::templates::{
    AboutTemplate, AdminCustomerApplicationsTemplate, AdminDashboardTemplate, AdminFixedDepositPlansTemplate, AdminFixedDepositsTemplate, AdminHomeLoansTemplate, AdminLoginTemplate, AdminPersonalLoansTemplate,
    BankingTemplate, ContactTemplate, CustomerActivityLogTemplate, CustomerPageTemplate, DashboardTemplate, DepositTemplate,
    ErrorTemplate, FaqTemplate, FixedDepositCreateTemplate, FixedDepositDashboardTemplate,
    ForbiddenTemplate, HomeLoanApplyTemplate, HomeLoanDashboardTemplate, HomeTemplate, LoginTemplate,
    LoanApplyTemplate, LoanDashboardTemplate, NotFoundTemplate, OnboardingFormTemplate,
    OnboardingFormTemplate1, OnboardingResultTemplate, OnboardingTemplate, ProfileTemplate,
    SecurityTemplate, SignupAccountTemplate, SignupContactTemplate, SignupEmploymentTemplate,
    SignupPersonalTemplate, SignupReviewTemplate, SignupSecurityTemplate, SignupTemplate,
    TransactionsTemplate, TransferTemplate,
};
