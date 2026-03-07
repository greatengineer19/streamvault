use anyhow::Context;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub port: u16,

    // Cloudflare R2
    pub r2_account_id: String,
    pub r2_access_key_id: String,
    pub r2_access_key: String,
    pub r2_bucket: String,

    // Public base URL of this API (used to build shareable links)
    pub public_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            port: std::env::var("PORT")
                    .unwrap_or_else(|_| "8080".into())
                    .parse()
                    .context("PORT must be a number")?,
            r2_account_id: required("R2_ACCOUNT_ID")?,
            r2_access_key_id: required("R2_ACCESS_KEY_ID")?,
            r2_secret_access_key: required("R2_SECRET_ACCESS_KEY")?,
            r2_bucket: required("R2_BUCKET")?,
            public_url: std::env::var("PUBLIC_URL")
                        .unwrap_or_else(|_| "http://localhost:8080".into()),
        })
    }
}

fn required(key: &str) -> anyhow::Result<String> {
    std::env::var(key).with_context(|| format!("Missing required env var: {key}"))
}

