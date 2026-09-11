//! Seed script to create or verify the initial administrator account.
//!
//! Usage:
//!     cargo run --bin seed
//!     cargo run --bin seed -- <email> <password> <name>
//!
//! Alternatively, configured via environment variables (.env):
//!     ADMIN_EMAIL=admin@internship.local
//!     ADMIN_PASSWORD=admin
//!     ADMIN_NAME="Admin User"

use internship_api::{
    config::Config,
    db::init_db,
    seed::seed_initial_admin,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "internship_api=info".into()),
        )
        .init();

    let mut config = Config::load()?;
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        config.admin_email = args[1].clone();
    }
    if args.len() > 2 {
        config.admin_password = args[2].clone();
    }
    if args.len() > 3 {
        config.admin_name = args[3..].join(" ");
    }

    println!("Connecting to database: {}", config.database_url);
    let db = init_db(&config).await?;

    println!("Ensuring initial administrator exists...");
    let admin = seed_initial_admin(&db, &config).await?;

    println!("\n==========================================");
    println!(" Administrator Ready");
    println!("==========================================");
    println!("  ID:        {}", admin.id);
    println!("  Name:      {}", admin.full_name);
    println!("  Email:     {}", admin.email);
    println!("  Role:      {:?}", admin.role);
    println!("==========================================\n");

    Ok(())
}
