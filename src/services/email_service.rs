use askama::DynTemplate;
use lettre::message::{header::ContentType, Message, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use std::{env, error::Error};

fn read_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn smtp_missing_message(to_email: &str, subject: &str, missing_key: &str) {
    eprintln!(
        "EMAIL NOT SENT: {missing_key} is not configured. To: {to_email}, Subject: {subject}"
    );
    eprintln!(
        "EMAIL SETUP: add SMTP_HOST, SMTP_PORT, SMTP_USERNAME, SMTP_PASSWORD and SMTP_FROM_EMAIL to .env, then restart cargo run."
    );
}

pub async fn send_template_email(
    to_email: &str,
    subject: &str,
    template_data: &dyn DynTemplate,
) -> Result<(), Box<dyn Error>> {
    let html_body = template_data.dyn_render()?;
    send_html_email(to_email, subject, html_body).await
}

pub async fn send_html_email(
    to_email: &str,
    subject: &str,
    html_body: String,
) -> Result<(), Box<dyn Error>> {
    let Some(smtp_host) = read_env("SMTP_HOST") else {
        smtp_missing_message(to_email, subject, "SMTP_HOST");
        return Ok(());
    };

    let Some(smtp_username) = read_env("SMTP_USERNAME") else {
        smtp_missing_message(to_email, subject, "SMTP_USERNAME");
        return Ok(());
    };

    let Some(smtp_password) = read_env("SMTP_PASSWORD") else {
        smtp_missing_message(to_email, subject, "SMTP_PASSWORD");
        return Ok(());
    };

    let smtp_from = read_env("SMTP_FROM_EMAIL").unwrap_or_else(|| smtp_username.clone());
    let smtp_port = read_env("SMTP_PORT")
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(587);

    let email = Message::builder()
        .from(smtp_from.parse()?)
        .to(to_email.parse()?)
        .subject(subject)
        .singlepart(
            SinglePart::builder()
                .header(ContentType::TEXT_HTML)
                .body(html_body),
        )?;

    let creds = Credentials::new(smtp_username.clone(), smtp_password);

    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)?
            .port(smtp_port)
            .credentials(creds)
            .build();

    println!(
        "EMAIL SEND ATTEMPT: smtp={smtp_host}:{smtp_port}, from={smtp_from}, username={smtp_username}, to={to_email}, subject={subject}"
    );

    match mailer.send(email).await {
        Ok(_) => {
            println!("EMAIL SENT: To: {to_email}, Subject: {subject}");
            Ok(())
        }
        Err(error) => {
            eprintln!("EMAIL FAILED: To: {to_email}, Subject: {subject}, Error: {error}");
            Err(Box::new(error))
        }
    }
}
