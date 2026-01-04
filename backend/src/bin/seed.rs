//! Seed script to create admin users and guests.
//!
//! Usage:
//!   cargo run --bin seed admin <username> <code>
//!   cargo run --bin seed guest <name> <party_size> <code>

use allmaptout_backend::auth::hash_password;
use rand::Rng;
use sqlx::postgres::PgPoolOptions;
use std::env;

fn generate_password() -> String {
    const CHARS: &[u8] = b"abcdefghijkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    (0..16)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("admin") => {
            let username = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("Usage: seed admin <username> <code>"))?;
            let code = args
                .get(3)
                .ok_or_else(|| anyhow::anyhow!("Usage: seed admin <username> <code>"))?;

            let pool = connect().await?;
            create_admin(&pool, username, code).await?;
        }
        Some("guest") => {
            let name = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("Usage: seed guest <name> <party_size> <code>"))?;
            let size: i32 = args
                .get(3)
                .ok_or_else(|| anyhow::anyhow!("Usage: seed guest <name> <party_size> <code>"))?
                .parse()?;
            let code = args
                .get(4)
                .ok_or_else(|| anyhow::anyhow!("Usage: seed guest <name> <party_size> <code>"))?;

            let pool = connect().await?;
            create_guest(&pool, name, size, code).await?;
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  cargo run --bin seed admin <username> <code>");
            eprintln!("  cargo run --bin seed guest <name> <party_size> <code>");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn connect() -> anyhow::Result<sqlx::PgPool> {
    let database_url = env::var("DATABASE_URL")?;
    Ok(PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?)
}

async fn create_admin(pool: &sqlx::PgPool, username: &str, code: &str) -> anyhow::Result<()> {
    let password = generate_password();
    let password_hash = hash_password(&password)?;

    sqlx::query(
        "INSERT INTO admins (username, password_hash) VALUES ($1, $2)
         ON CONFLICT (username) DO UPDATE SET password_hash = $2",
    )
    .bind(username)
    .bind(&password_hash)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO invite_codes (code, code_type) VALUES ($1, 'admin')
         ON CONFLICT (code) DO NOTHING",
    )
    .bind(code)
    .execute(pool)
    .await?;

    println!("Admin created:");
    println!("  Username: {}", username);
    println!("  Password: {}", password);
    println!("  Code:     {}", code);
    Ok(())
}

async fn create_guest(
    pool: &sqlx::PgPool,
    name: &str,
    party_size: i32,
    code: &str,
) -> anyhow::Result<()> {
    let guest_id: uuid::Uuid =
        sqlx::query_scalar("INSERT INTO guests (name, party_size) VALUES ($1, $2) RETURNING id")
            .bind(name)
            .bind(party_size)
            .fetch_one(pool)
            .await?;

    sqlx::query(
        "INSERT INTO invite_codes (code, code_type, guest_id) VALUES ($1, 'guest', $2)
         ON CONFLICT (code) DO NOTHING",
    )
    .bind(code)
    .bind(guest_id)
    .execute(pool)
    .await?;

    println!("Guest created:");
    println!("  Name:       {}", name);
    println!("  Party size: {}", party_size);
    println!("  Code:       {}", code);
    Ok(())
}
