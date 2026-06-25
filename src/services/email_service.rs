use crate::Config;
use lettre::message::{header::ContentType, Message, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Transport, AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use std::env;
use askama::DynTemplate;

pub async fn send_template_email(to_email: &str, subject: &str, template_data: &dyn DynTemplate) -> Result<(), Box<dyn std::error::Error>> {
    println!("this ran");
    let config = Config::from_env();

    let html_body = template_data.dyn_render()?;


    let email = Message::builder()
    .from(env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set in .env").parse()?)
    .to(to_email.parse()?)
    .subject(subject)
    .singlepart(
        SinglePart::builder()
        .header(ContentType::TEXT_HTML)
        .body(html_body),
    )?;

    println!("here");

    let creds = Credentials::new(env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set in .env"), env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set in .env"));

    let mailer: AsyncSmtpTransport<Tokio1Executor> = 
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&env::var("SMTP_HOST").expect("SMTP_HOST must be set in .env"))?
            .port(config.smtp_port)
            .credentials(creds)
            .build();

    println!("hm");

    mailer.send(email).await?;
    println!("ok");
    Ok(())
}