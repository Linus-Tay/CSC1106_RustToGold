use crate::controllers;
use actix_web::{guard, web};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .guard(guard::Host("apply.localhost"))
            .route("/onboarding/{path}", web::get().to(controllers::onboarding))
            .service(
                web::scope("/api")
                    .route("/onboarding/account", web::post().to(controllers::step1_post))
                    .route("/onboarding/personal", web::post().to(controllers::step2_post))
                    .route("/onboarding/contact", web::post().to(controllers::step3_post))
                    .route("/onboarding/employment", web::post().to(controllers::step4_post))
                    .route("/onboarding/submit", web::post().to(controllers::submit)),
            ),
    )
    .service(
        web::scope("")
            .route("/", web::get().to(controllers::home))
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
                    //.route("/submit", web::post().to(controllers::post_signup_submit)),
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
                    .route("/loans", web::get().to(controllers::loans_page))
                    .route("/loans/apply", web::get().to(controllers::loan_apply_page))
                    .route("/fixed-deposits", web::get().to(controllers::fixed_deposits_page))
                    .route("/fixed-deposits/new", web::get().to(controllers::fixed_deposit_new_page))
                    .route("/profile", web::get().to(controllers::profile_page))
                    .route("/profile", web::post().to(controllers::update_profile))
                    .route("/approve/{path}", web::post().to(controllers::approve_customer_with_product))
            ),
    );
}
