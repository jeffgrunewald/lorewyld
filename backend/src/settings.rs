use anyhow::Result;
use config::{Config, Environment, File};
use jwt_simple::prelude::Ed25519KeyPair;
use serde::Deserialize;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::{fs, net::SocketAddr, path::Path, str::FromStr};

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default = "default_log")]
    pub log: String,

    #[serde(default = "default_listen")]
    pub api_addr: String,

    #[serde(default = "default_db_path")]
    pub db_path: String,
    pub db_max_connections: u32,

    #[serde(default = "default_key_path")]
    pub jwt_key_path: String,
}

fn default_log() -> String {
    "lorewyld=info".to_string()
}

fn default_listen() -> String {
    "0.0.0.0:8080".to_string()
}

fn default_db_path() -> String {
    "database.db".to_string()
}

fn default_key_path() -> String {
    "jwt_signing_key.bin".to_string()
}

impl Settings {
    pub fn new<P: AsRef<Path>>(path: Option<P>) -> Result<Self> {
        let mut builder = Config::builder();

        if let Some(file) = path {
            builder = builder
                .add_source(File::with_name(&file.as_ref().to_string_lossy()).required(false))
        }

        builder
            .add_source(Environment::with_prefix("LW").separator("__"))
            .build()
            .and_then(|config| config.try_deserialize())
            .map_err(anyhow::Error::from)
    }

    pub fn listen_addr(&self) -> Result<SocketAddr> {
        SocketAddr::from_str(&self.api_addr).map_err(anyhow::Error::from)
    }

    pub async fn db_connect(&self) -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(self.db_max_connections)
            .connect(&format!("sqlite:{}?mode=rwc", self.db_path))
            .await
            .expect("Failed to create pool")
    }

    pub fn jwt_keypair(&self) -> Result<Ed25519KeyPair> {
        let path = Path::new(&self.jwt_key_path);

        // Try to read existing key file
        if path.exists() {
            let bytes = fs::read(path)?;
            if !bytes.is_empty() {
                return Ed25519KeyPair::from_bytes(&bytes)
                    .map_err(|e| anyhow::anyhow!("Failed to parse JWT keypair: {}", e));
            }
        }

        // File doesn't exist or is empty - create new keypair
        let keypair = Ed25519KeyPair::generate();
        fs::write(path, keypair.to_bytes())?;
        Ok(keypair)
    }
}
