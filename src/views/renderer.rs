// View layer: Askama template structs and rendering helpers.

use actix_web::{error::ErrorInternalServerError, HttpResponse, Result};
use askama::Template;

// Renders an Askama template into an HTTP response.
pub fn render<T: Template>(template: T) -> Result<HttpResponse> {
    let html = template.render().map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

// Renders an Askama template into an HTTP response.
pub fn render_html(html: String) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}
