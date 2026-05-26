pub mod api;
pub mod settings;
pub mod web;

use anyhow::Result;
use rand::Rng;
use sqlx::SqlitePool;
use uuid::Uuid;

const JOIN_CODE_ALPHABET: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const JOIN_CODE_BLOCK_LEN: usize = 6;
const JOIN_CODE_BLOCK_COUNT: usize = 3;

pub fn generate_join_code() -> String {
    let mut rng = rand::rng();
    (0..JOIN_CODE_BLOCK_COUNT)
        .map(|_| {
            (0..JOIN_CODE_BLOCK_LEN)
                .map(|_| {
                    let idx = rng.random_range(0..JOIN_CODE_ALPHABET.len());
                    JOIN_CODE_ALPHABET[idx] as char
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("-")
}

pub async fn get_server_name(db: &SqlitePool) -> Result<String> {
    sqlx::query_scalar("SELECT name FROM game_server LIMIT 1")
        .fetch_one(db)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn setup_game_server(db: &SqlitePool) -> Result<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM game_server)")
        .fetch_one(db)
        .await?;
    if exists {
        return Ok(());
    }

    let id = Uuid::new_v4().to_string();
    let version = env!("CARGO_PKG_VERSION");
    let join_code = generate_join_code();

    sqlx::query("INSERT INTO game_server (id, name, version, join_code) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind("Lorewyld")
        .bind(version)
        .bind(&join_code)
        .execute(db)
        .await?;

    Ok(())
}
