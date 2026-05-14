use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub session_secret: String,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
        let session_secret = env::var("SESSION_SECRET").expect("SESSION_SECRET must be set in .env");
        let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("SERVER_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(3000);

        if session_secret.as_bytes().len() < 64 {
            panic!("SESSION_SECRET must be at least 64 bytes for cookie session security.");
        }

        Self {
            database_url,
            session_secret,
            host,
            port,
        }
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
