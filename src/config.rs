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
            ghost_url: std::env::var("GHOST_URL")
                .map_err(|_| anyhow::anyhow!("GHOST_URL environment variable not found"))?,
            ghost_admin_id: std::env::var("GHOST_ADMIN_ID")
                .map_err(|_| anyhow::anyhow!("GHOST_ADMIN_ID environment variable not found"))?,
            ghost_admin_secret: std::env::var("GHOST_ADMIN_SECRET")
                .map_err(|_| anyhow::anyhow!("GHOST_ADMIN_SECRET environment variable not found"))?,
            webhook_secret: std::env::var("WEBHOOK_SECRET")
                .map_err(|_| anyhow::anyhow!("WEBHOOK_SECRET environment variable not found"))?,
            resend_api_key: std::env::var("RESEND_API_KEY")
                .map_err(|_| anyhow::anyhow!("RESEND_API_KEY environment variable not found"))?,
            from_email: std::env::var("FROM_EMAIL")
                .map_err(|_| anyhow::anyhow!("FROM_EMAIL environment variable not found"))?,
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("PORT must be a valid number"))?,
        })
    }
}

