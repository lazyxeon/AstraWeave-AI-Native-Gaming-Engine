use clap::{Parser, Subcommand};
use astraweave_secrets::{SecretManager, SecretValue};

#[derive(Parser)]
#[command(name = "aw_secrets")]
#[command(about = "AstraWeave secrets management CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Store a secret
    Set { key: String },
    /// Retrieve a secret
    Get { key: String },
    /// Delete a secret
    Delete { key: String },
    /// List all secret keys
    List,
    /// Interactive setup
    Init,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let manager = SecretManager::global();
    
    match cli.command {
        Commands::Set { key } => {
            println!("Enter secret value (hidden):");
            let value = rpassword::read_password()?;
            manager.set(&key, SecretValue::from_str(&value))?;
            println!("[OK] Secret stored: {}", key);
        }
        Commands::Get { key } => {
            let value = manager.get(&key)?;
            println!("{}", value.as_str()?);
        }
        Commands::Delete { key } => {
            manager.delete(&key)?;
            println!("[OK] Secret deleted: {}", key);
        }
        Commands::List => {
            println!("Note: Listing not yet implemented");
        }
        Commands::Init => {
            interactive_init(manager)?;
        }
    }
    
    Ok(())
}

fn interactive_init(manager: &SecretManager) -> anyhow::Result<()> {
    println!("AstraWeave Secrets Setup");
    println!("========================\n");
    
    // LLM API Key
    println!("OpenAI/LLM API Key (optional, press Enter to skip):");
    if let Ok(key) = rpassword::read_password() {
        if !key.is_empty() {
            manager.set("llm.api_key", SecretValue::from_str(&key))?;
            println!("[OK] Stored llm.api_key\n");
        }
    }
    
    println!("Setup complete!");
    Ok(())
}
