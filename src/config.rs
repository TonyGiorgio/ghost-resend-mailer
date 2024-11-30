use anyhow::Result;

#[derive(Clone)]
pub struct Config {
    pub ghost_url: String,
    pub ghost_admin_id: String,
    pub ghost_admin_secret: String,
    pub webhook_secret: String,
    pub resend_api_key: String,
    pub from_email: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        Ok(Config {
            ghost_url: std::env::var("GHOST_URL")?,
            ghost_admin_id: std::env::var("GHOST_ADMIN_ID")?,
            ghost_admin_secret: std::env::var("GHOST_ADMIN_SECRET")?,
            webhook_secret: std::env::var("WEBHOOK_SECRET")?,
            resend_api_key: std::env::var("RESEND_API_KEY")?,
            from_email: std::env::var("FROM_EMAIL")?,
            port: std::env::var("PORT")?.parse()?,
        })
    }
}

