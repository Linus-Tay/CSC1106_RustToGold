use crate::controllers;
use actix_web::{web, guard};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::scope("")
        .guard(guard::Host("apply.localhost"))
        .route("/onboarding/{path}", web::get().to(controllers::onboarding))
        .service(web::scope("/api")
            .route("/onboarding/actions/submit-step1", web::post().to(controllers::step1_post))
            .route("/onboarding/actions/submit", web::post().to(controllers::submit))
        )
        //.route("/onboarding/product-information", web::get().to(controllers::display_product))
        //.route("/onboarding/init", web::get().to(controllers::redirect_to_product_information))
        //.route("/onboarding/primary-contact-details", web::get().to(controllers::show_form))
    )
    .service(
        web::scope("")
        .route("/", web::get().to(controllers::home))
        .route("/login", web::get().to(controllers::login_page))
        .route("/login", web::post().to(controllers::login))
        .route("/signup", web::get().to(controllers::signup_page))
        .route("/signup", web::post().to(controllers::signup))
        .route("/logout", web::get().to(controllers::logout))
        .route("/403", web::get().to(controllers::forbidden))
        .service(
            web::scope("/customer")
                .route("/dashboard", web::get().to(controllers::dashboard))
                .route("/deposit", web::get().to(controllers::deposit_page))
                .route("/deposit", web::post().to(controllers::deposit))
                .route("/transfer", web::get().to(controllers::transfer_page))
                .route("/transactions", web::get().to(controllers::transactions))
                .route("/loans", web::get().to(controllers::loans_page))
                .route("/loans/apply", web::get().to(controllers::loan_apply_page))
                .route("/fixed-deposits", web::get().to(controllers::fixed_deposits_page))
                .route("/fixed-deposits/new", web::get().to(controllers::fixed_deposit_new_page))
                .route("/profile", web::get().to(controllers::profile_page))
                .route("/profile", web::post().to(controllers::update_profile)),
        )
    );

}
