// Controller layer: handles HTTP/session flow and delegates business rules to services.

use crate::models::User;
use crate::repositories::user_repository;
use crate::AppState;
use actix_session::Session;
use actix_web::{http::header, web, HttpResponse, Result};
use uuid::Uuid;

const CUSTOMER_USER_ID_KEY: &str = "customer_user_id";
const CUSTOMER_ROLE_KEY: &str = "customer_role";
const ADMIN_USER_ID_KEY: &str = "admin_user_id";
const ADMIN_ROLE_KEY: &str = "admin_role";

// Handles the require customer request.
pub async fn require_customer(
    data: &web::Data<AppState>,
    session: &Session,
) -> Result<User, HttpResponse> {
    let user = require_active_user(data, session, CUSTOMER_USER_ID_KEY, "/login").await?;

    if !user.is_customer() {
        clear_customer_session(session);
        return Err(redirect("/login"));
    }

    Ok(user)
}

// Handles the require admin request.
pub async fn require_admin(
    data: &web::Data<AppState>,
    session: &Session,
) -> Result<User, HttpResponse> {
    let user = require_active_user(data, session, ADMIN_USER_ID_KEY, "/admin/login").await?;

    if !user.is_staff_or_admin() {
        clear_admin_session(session);
        return Err(redirect("/admin/login"));
    }

    Ok(user)
}

// Handles the require active user request.
async fn require_active_user(
    data: &web::Data<AppState>,
    session: &Session,
    key: &str,
    login_path: &str,
) -> Result<User, HttpResponse> {
    let Some(user_id) = session_uuid(session, key) else {
        clear_session_key(session, key);
        return Err(redirect(login_path));
    };

    let user = match user_repository::find_user_by_id(&data.db, user_id).await {
        Ok(Some(user)) => user,
        _ => {
            clear_session_key(session, key);
            return Err(redirect(login_path));
        }
    };

    if !user.is_active() {
        clear_session_key(session, key);
        return Err(redirect(login_path));
    }

    Ok(user)
}

// Handles customer session user id session flow.
pub fn customer_session_user_id(session: &Session) -> Option<Uuid> {
    session_uuid(session, CUSTOMER_USER_ID_KEY)
}

// Handles admin session user id session flow.
pub fn admin_session_user_id(session: &Session) -> Option<Uuid> {
    session_uuid(session, ADMIN_USER_ID_KEY)
}

// Handles store customer session session flow.
pub fn store_customer_session(session: &Session, user: &User) -> Result<()> {
    session.insert(CUSTOMER_USER_ID_KEY, user.id.to_string())?;
    session.insert(CUSTOMER_ROLE_KEY, user.role.clone())?;
    Ok(())
}

// Handles store admin session session flow.
pub fn store_admin_session(session: &Session, user: &User) -> Result<()> {
    session.insert(ADMIN_USER_ID_KEY, user.id.to_string())?;
    session.insert(ADMIN_ROLE_KEY, user.role.clone())?;
    Ok(())
}

// Handles clear customer session session flow.
pub fn clear_customer_session(session: &Session) {
    session.remove(CUSTOMER_USER_ID_KEY);
    session.remove(CUSTOMER_ROLE_KEY);
}

// Handles clear admin session session flow.
pub fn clear_admin_session(session: &Session) {
    session.remove(ADMIN_USER_ID_KEY);
    session.remove(ADMIN_ROLE_KEY);
}

// Handles session uuid session flow.
fn session_uuid(session: &Session, key: &str) -> Option<Uuid> {
    session
        .get::<String>(key)
        .ok()
        .flatten()
        .and_then(|value| Uuid::parse_str(&value).ok())
}

// Handles clear session key session flow.
fn clear_session_key(session: &Session, key: &str) {
    if key == CUSTOMER_USER_ID_KEY {
        clear_customer_session(session);
    } else if key == ADMIN_USER_ID_KEY {
        clear_admin_session(session);
    } else {
        session.remove(key);
    }
}


// Handles the redirect request.
pub fn redirect(path: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .append_header((header::LOCATION, path))
        .finish()
}
