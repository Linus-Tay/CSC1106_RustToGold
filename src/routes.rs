use crate::controllers;
use actix_web::{guard, web};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .guard(guard::Host("apply.localhost"))
            .route("/onboarding/{path}", web::get().to(controllers::onboarding))
            .service(
                web::scope("/api")
                    .route(
                        "/onboarding/actions/submit-step1",
                        web::post().to(controllers::step1_post),
                    )
                    .route(
                        "/onboarding/actions/submit",
                        web::post().to(controllers::submit),
                    ),
            ),
    )
    .service(
        web::scope("")
            .route("/", web::get().to(controllers::home))
            .route("/banking", web::get().to(controllers::banking_page))
            .route("/security", web::get().to(controllers::security_page))
            .route("/about", web::get().to(controllers::about_page))
            .route("/faq", web::get().to(controllers::faq_page))
            .route("/contact", web::get().to(controllers::contact_page))
            .route("/login", web::get().to(controllers::login_page))
            .route("/login", web::post().to(controllers::login))
            .route("/signup", web::get().to(controllers::signup_page))
            .route("/signup", web::post().to(controllers::signup))
            .service(
                web::scope("/signup")
                    .route("/account", web::get().to(controllers::show_signup_account))
                    .route("/account", web::post().to(controllers::post_signup_account))
                    .route("/personal", web::get().to(controllers::show_signup_personal))
                    .route("/personal", web::post().to(controllers::post_signup_personal))
                    .route("/contact", web::get().to(controllers::show_signup_contact))
                    .route("/contact", web::post().to(controllers::post_signup_contact))
                    .route("/employment", web::get().to(controllers::show_signup_employment))
                    .route("/employment", web::post().to(controllers::post_signup_employment))
                    .route("/security", web::get().to(controllers::show_signup_security))
                    .route("/security", web::post().to(controllers::post_signup_security))
                    .route("/review", web::get().to(controllers::show_signup_review))
                    .route("/submit", web::post().to(controllers::post_signup_submit)),
            )
            .route("/logout", web::get().to(controllers::logout))
            .route("/403", web::get().to(controllers::forbidden))
            .service(
                web::scope("/customer")
                    .route("/dashboard", web::get().to(controllers::dashboard))
                    .route("/deposit", web::get().to(controllers::deposit_page))
                    .route("/deposit", web::post().to(controllers::deposit))
                    .route("/transfer", web::get().to(controllers::transfer_page))
                    .route("/transfer", web::post().to(controllers::transfer))
                    .route("/transactions", web::get().to(controllers::transactions))
                    .route("/loan-activity", web::get().to(controllers::loan_activity))
                    .route("/loan-log", web::get().to(controllers::loan_activity))
                    .route("/fixed-deposit-activity", web::get().to(controllers::fixed_deposit_activity))
                    .route("/fixed-deposit-log", web::get().to(controllers::fixed_deposit_activity))
                    .route("/loans", web::get().to(controllers::loans_page))
                    .route("/loans/apply", web::get().to(controllers::loan_apply_page))
                    .route("/loans/apply", web::post().to(controllers::create_personal_loan))
                    .route("/loans/{id}/pay", web::post().to(controllers::pay_loan))
                    .route("/home-loans", web::get().to(controllers::home_loans_page))
                    .route("/home-loans/apply", web::get().to(controllers::home_loan_apply_page))
                    .route("/home-loans/apply", web::post().to(controllers::create_home_loan_application))
                    .route("/home-loans/{id}/pay", web::post().to(controllers::pay_home_loan))
                    .route("/fixed-deposits", web::get().to(controllers::fixed_deposits_page))
                    .route("/fixed-deposits", web::post().to(controllers::create_fixed_deposit))
                    .route("/fixed-deposits/new", web::get().to(controllers::fixed_deposit_new_page))
                    .route("/fixed-deposits/{id}/withdraw", web::post().to(controllers::withdraw_fixed_deposit))
                    .route("/profile", web::get().to(controllers::profile_page))
                    .route("/profile", web::post().to(controllers::update_profile))
                    .route("/approve/{path}", web::post().to(controllers::approve_product)),
            )
            .service(
                web::scope("/admin")
                    .route("/login", web::get().to(controllers::admin_login_page))
                    .route("/login", web::post().to(controllers::admin_login))
                    .route("/logout", web::get().to(controllers::admin_logout))
                    .route("", web::get().to(controllers::admin_dashboard))
                    .route("/dashboard", web::get().to(controllers::admin_dashboard))
                    .route("/signups", web::get().to(controllers::admin_signups_page))
                    .route("/signups/{id}/approve", web::post().to(controllers::approve_customer_application))
                    .route("/signups/{id}/reject", web::post().to(controllers::reject_customer_application))
                    .route("/personal-loans", web::get().to(controllers::admin_personal_loans_page))
                    .route("/personal-loans/{id}/approve", web::post().to(controllers::approve_personal_loan))
                    .route("/personal-loans/{id}/reject", web::post().to(controllers::reject_personal_loan))
                    .route("/home-loans", web::get().to(controllers::admin_home_loans_page))
                    .route("/home-loans/{id}/approve", web::post().to(controllers::approve_home_loan))
                    .route("/home-loans/{id}/reject", web::post().to(controllers::reject_home_loan))
                    .route("/fixed-deposits", web::get().to(controllers::admin_fixed_deposits_page))
                    .route("/fixed-deposit-plans", web::get().to(controllers::admin_fixed_deposit_plans_page))
                    .route("/fixed-deposit-plans", web::post().to(controllers::create_fixed_deposit_plan))
                    .route("/fixed-deposit-plans/{id}", web::post().to(controllers::update_fixed_deposit_plan)),
            ),
    );
}
