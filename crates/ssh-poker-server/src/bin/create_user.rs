use anyhow::Result;
use argon2::{Argon2, PasswordHasher, password_hash::{rand_core::OsRng, SaltString}};
use data_store::{Database, DatabaseConfig};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <username> <password> [email] [database]", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} test test123", args[0]);
        eprintln!("  {} admin admin123 admin@example.com", args[0]);
        std::process::exit(1);
    }
    
    let username = &args[1];
    let password = &args[2];
    let email = if args.len() > 3 { Some(args[3].clone()) } else { None };
    let db_path = if args.len() > 4 { args[4].clone() } else { "poker_game.db".to_string() };
    
    // Connect to database
    let db_config = DatabaseConfig {
        database_path: db_path.clone(),
        create_if_missing: true,
        max_connections: 1,
    };
    
    println!("Connecting to database: {}", db_path);
    let database = Database::new(db_config).await?;
    
    // Generate password hash using Argon2
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();
    
    // Create user
    let user = data_store::models::NewUser {
        username: username.to_string(),
        email: email.clone(),
        password_hash,
    };
    
    match database.create_user(user).await {
        Ok(created_user) => {
            println!("✅ User created successfully!");
            println!("  ID: {}", created_user.id);
            println!("  Username: {}", created_user.username);
            if let Some(email) = &created_user.email {
                println!("  Email: {}", email);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to create user: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}