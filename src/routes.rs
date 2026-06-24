use crate::controllers;
use actix_web::{web, guard};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::scope("")
        .guard(guard::Host("apply.localhost"))
        .route("/onboarding/init", web::get().to(controllers::display_product))
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
                .route("/fixed-deposits", web::post().to(controllers::create_fixed_deposit))
                .route("/fixed-deposits/{id}/withdraw", web::post().to(controllers::withdraw_fixed_deposit))
                .route("/profile", web::get().to(controllers::profile_page))
                .route("/profile", web::post().to(controllers::update_profile)),
        )
        .service(
            web::scope("/admin")
                //.route("/dashboard", web::get().to(controllers::admin_dashboard))
                .service(
                    web::resource("/staff")
                        .route(web::get().to(controllers::staff_controller::admin_staff_page))
                        .route(web::post().to(controllers::staff_controller::create_staff)),
                )
                .service(
                    web::resource("/staff/new")
                        .route(web::get().to(controllers::staff_controller::admin_staff_new_page)),
                )
                .service(
                    web::resource("/staff/{id}/edit")
                        .route(web::get().to(controllers::staff_controller::admin_staff_edit_page)),
                )
                .service(
                    web::resource("/staff/{id}")
                        .route(web::post().to(controllers::staff_controller::update_staff)),
                )
                .route("/audit-log", web::get().to(controllers::admin_audit_log_page))
                .service(
                    web::resource("/staff/{id}/delete")
                        .route(web::post().to(controllers::staff_controller::delete_staff)),
                )
                .route("/transactions", web::get().to(controllers::admin_account_controller::admin_transactions_page))
                .service(
                    web::resource("/accounts")
                        .route(web::get().to(controllers::admin_account_controller::admin_accounts_page)),
                )
                .service(
                    web::resource("/accounts/{id}/approve")
                        .route(web::post().to(controllers::admin_account_controller::approve_account)),
                )
                .service(
                    web::resource("/accounts/{id}/freeze")
                        .route(web::post().to(controllers::admin_account_controller::freeze_account)),
                )
                .service(
                    web::resource("/accounts/{id}/close")
                        .route(web::post().to(controllers::admin_account_controller::close_account)),
                )
                .route("/fixed-deposits", web::get().to(controllers::admin_fixed_deposits_page))
                .route("/fixed-deposit-plans", web::get().to(controllers::admin_fixed_deposit_plans_page))
                .route("/fixed-deposit-plans", web::post().to(controllers::create_fixed_deposit_plan))
                .route("/fixed-deposit-plans/{id}", web::post().to(controllers::update_fixed_deposit_plan)),
        )
    );

}
