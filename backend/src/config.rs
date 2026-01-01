use std::env;

use anyhow::{Context, Result};

pub struct Config {
    pub port: u16,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3001".into())
                .parse()
                .context("PORT must be a number")?,
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required")?,
        })
    }
}
