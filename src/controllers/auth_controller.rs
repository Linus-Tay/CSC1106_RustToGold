use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{
    admin_session_user_id, clear_admin_session, clear_customer_session, customer_session_user_id,
    redirect, store_admin_session, store_customer_session,
};
use crate::forms::LoginForm;
use crate::forms::auth_forms::TwoFactorForm;
use crate::repositories::user_repository;
use crate::services::{self, authenticate_device};
use crate::views::templates::TwoFactorAuthTemplate;
use crate::views::{AdminLoginTemplate, LoginTemplate, NotFoundTemplate, render};
use crate::AppState;
use actix_session::Session;
use actix_web::cookie::Cookie;
use actix_web::http::header;
use actix_web::{web, HttpResponse, Result, HttpRequest};

// Renders the login page
pub async fn login_page(session: Session) -> Result<HttpResponse> {
    if customer_session_user_id(&session).is_some() {
        return Ok(redirect("/customer/dashboard"));
    }

    render(LoginTemplate {
        error: String::new(),
        has_error: false,
    })
}

// Renders the admin login page
pub async fn admin_login_page(session: Session) -> Result<HttpResponse> {
    if admin_session_user_id(&session).is_some() {
        return Ok(redirect("/admin/dashboard"));
    }

    render(AdminLoginTemplate {
        error: String::new(),
        has_error: false,
    })
}

// Handles login session flow.
pub async fn login(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<LoginForm>,
    req: HttpRequest // <-- FIXED: Changed from HttpResponse to HttpRequest
) -> Result<HttpResponse> {
    match services::authenticate_user(&data.db, form.into_inner()).await {
        Ok(user) if user.is_customer() => {
            if let Some(cookie) = req.cookie("device_id") {
                let raw_token = cookie.value();
                println!("{}", raw_token);
                if let Ok(_device) = authenticate_device(&data.db, raw_token).await {
                    store_customer_session(&session, &user)?;
                    return Ok(redirect("/customer/dashboard")); 
                }
            }

            match services::generate_and_send_2fa(&data.db, &user.id).await {
                Ok(()) => (),
                Err(e) => {
                    return render_error("Two Factor Authentication", "Unable to send 2FA".to_string())
                }
            };

            session.insert("pending_2fa_user_id", user.id.to_string())?;
            Ok(redirect("/2fa"))
        }
        Ok(_) => render(LoginTemplate {
            error: "Use the admin login page for staff access.".to_string(),
            has_error: true,
        }),
        Err(error) => render(LoginTemplate {
            error,
            has_error: true,
        }),
    }
}

// Renders the two factor page
pub async fn twofactor_page(session: Session) -> Result<HttpResponse> {
    if let Ok(Some(pending_user_id)) = session.get::<String>("pending_2fa_user_id") {
        println!("{}", pending_user_id);
        render(TwoFactorAuthTemplate {
            error: String::new(),
            has_error: false,
        })
    }
    else {
        render(NotFoundTemplate)
    }
}

// Handles 2fa verification
pub async fn verify_2fa(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<TwoFactorForm>,
    req: HttpRequest
) -> Result<HttpResponse> {

    if let Ok(Some(pending_user_id)) = session.get::<String>("pending_2fa_user_id") {
        let form = form.into_inner();
        let user_id = match uuid::Uuid::parse_str(&pending_user_id) {
            Ok(value) => value,
            Err(_) => return render_error("Two Factor Authentication", "Unexpected error occured".to_string())

        };

        match services::verify_2fa(&data.db, &form.code, &user_id).await {
        Ok(()) => {
                let Ok(Some(user)) = user_repository::find_user_by_id(&data.db, user_id).await else {
                    return render_error("Two Factor Authentication", "Unable to verify code".to_string())

                };
                
                store_customer_session(&session, &user)?;
                session.remove("pending_2fa_user_id");

                let raw_token = uuid::Uuid::new_v4().to_string();
                if let Err(error) = services::add_trusted_device(&data.db, &user.id, &raw_token).await {
                    eprintln!("Failed to save trusted device for user {}: {}", user.id, error);
                }

                let my_cookie = Cookie::build("device_id", raw_token)
                .path("/") // Makes it available across the whole site
                .secure(false) // Only send over HTTPS
                .http_only(true) // Protects from XSS
                .finish();

                // 3. Build the redirect response and attach the cookie
                return Ok(HttpResponse::Found()
                    .append_header((header::LOCATION, "/customer/dashboard"))
                    .cookie(my_cookie)
                    .finish());

                //return Ok(redirect("/customer/dashboard")); 
            }
            Err(error) => render(TwoFactorAuthTemplate {
                error,
                has_error: true,
            }),
        }
    }
    else {
        return render_error("Two Factor Authentication", "Unexpected error occured".to_string())

    }
}

// Handles admin login session flow.
pub async fn admin_login(
    data: web::Data<AppState>,
    session: Session,
    form: web::Form<LoginForm>,
) -> Result<HttpResponse> {
    match services::authenticate_user(&data.db, form.into_inner()).await {
        Ok(user) if user.is_staff_or_admin() => {
            store_admin_session(&session, &user)?;
            Ok(redirect("/admin/dashboard"))
        }
        Ok(_) => render(AdminLoginTemplate {
            error: "This login is only for staff and admin users.".to_string(),
            has_error: true,
        }),
        Err(error) => render(AdminLoginTemplate {
            error,
            has_error: true,
        }),
    }
}

// Handles logout session flow.
pub async fn logout(session: Session) -> Result<HttpResponse> {
    clear_customer_session(&session);
    Ok(redirect("/"))
}

// Handles admin logout session flow.
pub async fn admin_logout(session: Session) -> Result<HttpResponse> {
    clear_admin_session(&session);
    Ok(redirect("/admin/login"))
}