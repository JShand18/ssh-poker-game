use wish_server::run_server;
use data_store::Database;
use clap::{Parser, ValueEnum};
use colored::*;
use log::{info, error};
use std::path::Path;

#[derive(Parser)]
#[command(name = "ssh-poker-server")]
#[command(about = "SSH-accessible multiplayer poker game server")]
#[command(version)]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "2222")]
    port: u16,
    
    /// Address to bind to
    #[arg(short, long, default_value = "0.0.0.0")]
    address: String,
    
    /// Database file path (SQLite)
    #[arg(short, long, default_value = "poker_game.db")]
    database: String,
    
    /// Create a demo user for testing
    #[arg(long)]
    create_demo_user: bool,
    
    /// Enable debug logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();
    
    // Print welcome banner
    print_banner();
    
    // Initialize database
    println!("{}", "🗄️  Initializing database...".cyan());
    let db_config = data_store::DatabaseConfig {
        database_path: cli.database.clone(),
        create_if_missing: true,
        max_connections: 10,
    };
    let database = Database::new(db_config).await?;
    
    // Create demo user if requested
    if cli.create_demo_user {
        println!("{}", "👤 Creating demo user...".cyan());
        create_demo_user(&database).await?;
    }
    
    // Start server
    println!("{}", format!("🚀 Starting SSH Poker Server on {}:{}", cli.address, cli.port).green().bold());
    println!("{}", "📋 Server Information:".yellow());
    println!("   • Database: {}", cli.database);
    println!("   • SSH Port: {}", cli.port);
    println!("   • Bind Address: {}", cli.address);
    println!();
    println!("{}", "🎮 How to connect:".yellow().bold());
    println!("   ssh -p {} <username>@{}", cli.port, if cli.address == "0.0.0.0" { "localhost" } else { &cli.address });
    println!();
    println!("{}", "📚 Available commands:".yellow());
    println!("   • (F)old, (C)all/Check, (R)aise, (A)ll-in");
    println!("   • (Q)uit to disconnect");
    println!();
    
    if cli.create_demo_user {
        println!("{}", "🎯 Demo User Created:".green().bold());
        println!("   Username: demo");
        println!("   Password: demo123");
        println!("   Try: ssh -p {} demo@{}", cli.port, if cli.address == "0.0.0.0" { "localhost" } else { &cli.address });
        println!();
    }
    
    info!("Starting SSH server on {}:{}", cli.address, cli.port);
    
    // Run the server
    if let Err(e) = run_server(database, &cli.address, cli.port).await {
        error!("Server error: {}", e);
        eprintln!("{} {}", "❌ Server failed:".red().bold(), e);
        std::process::exit(1);
    }
    
    Ok(())
}

fn print_banner() {
    println!("{}", r#"
╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║    ♠♥♦♣  SSH POKER GAME SERVER  ♣♦♥♠                                          ║
║                                                                              ║
║    A terminal-based multiplayer Texas Hold'em poker game                    ║
║    accessible via any SSH client with AI opponents                          ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
"#.green());
}

async fn create_demo_user(database: &Database) -> Result<(), Box<dyn std::error::Error>> {
    use wish_server::SecureAuthService;
    
    let mut auth_service = SecureAuthService::new(database.clone());
    
    match auth_service.create_user("demo", "demo123", "demo@example.com").await {
        Ok(user_id) => {
            println!("   ✅ Demo user created with ID: {}", user_id);
        }
        Err(e) => {
            println!("   ⚠️  Demo user may already exist: {}", e);
        }
    }
    
    Ok(())
} 