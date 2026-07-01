pub mod renderer;
pub mod templates;

pub use self::renderer::render;
pub use self::templates::{
    AboutTemplate, AccountCreationEmailTemplate, AccountCreationSetupTemplate, AccountCreationSuccessTemplate, ApplicationReceivedEmailTemplate, OnboardingAccountTemplate, OnboardingContactTemplate, OnboardingEmploymentTemplate, OnboardingPersonalTemplate, OnboardingReviewTemplate, AdminAuditLogTemplate, AdminCustomerAccountsTemplate, AdminCustomerApplicationsTemplate, AdminDashboardTemplate, AdminFixedDepositPlansTemplate, AdminFixedDepositsTemplate, AdminHomeLoansTemplate, AdminLoginTemplate, AdminPersonalLoansTemplate, AdminStaffTemplate,
    BankingTemplate, ContactTemplate, CardDashboardTemplate, CustomerActivityLogTemplate, CustomerPageTemplate, DashboardTemplate, DepositTemplate,
    ErrorTemplate, FaqTemplate, FixedDepositCreateTemplate, FixedDepositDashboardTemplate,
    ForbiddenTemplate, HomeLoanApplyTemplate, HomeLoanDashboardTemplate, HomeTemplate, LoginTemplate,
    LoanApplyTemplate, LoanDashboardTemplate, NotFoundTemplate, OnboardingResultTemplate, OnboardingTemplate, PayNowTemplate, ProfileTemplate,
    SecurityTemplate, TransactionsTemplate, TransferTemplate,
};
