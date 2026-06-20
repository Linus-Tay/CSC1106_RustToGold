use crate::controllers::error_controller::render_error;
use crate::controllers::session_guard::{redirect, require_admin, require_customer};
use crate::forms::{};
use crate::services;
use crate::views::render;
use crate::views::templates::{};
use crate::AppState;
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};

