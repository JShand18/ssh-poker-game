use ssh_server::{ServerConfig, run_server};
use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "ssh-poker-server")]
#[command(about = "SSH-accessible multiplayer poker game server")]
struct Cli {
    /// Server mode
    #[arg(value_enum, default_value = "simple")]
    mode: ServerMode,
    
    /// Port to listen on
    #[arg(short, long, default_value = "2222")]
    port: u16,
    
    /// Address to bind to
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ServerMode {
    /// Simple text-based interface
    Simple,
    /// Rich terminal UI with graphics
    Tui,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    let config = ServerConfig::new()
        .with_address(cli.address.clone())
        .with_port(cli.port);
    
    match cli.mode {
        ServerMode::Simple => {
            println!("🚀 Starting Poker Server (Simple Mode) on {}:{}", config.address, config.port);
            println!("Connect with one of:");
            println!("  - telnet {} {}", config.address, config.port);
            println!("  - nc {} {}", config.address, config.port);
            println!("  - ssh -p {} <username>@{} (coming soon)\n", config.port, config.address);
            
            run_server(config).await?;
        }
        ServerMode::Tui => {
            println!("🚀 Starting Poker Server (TUI Mode) on {}:{}", config.address, config.port);
            println!("Connect with: nc {} {}", config.address, config.port);
            println!("Note: Use a terminal that supports ANSI escape sequences\n");
            
            ssh_server::tui_server::run_tui_server(config).await?;
        }
    }
    
    Ok(())
} 