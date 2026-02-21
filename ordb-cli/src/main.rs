mod cli;
mod db;

mod api_client;
mod phases;
mod scanner;
mod metadata;
mod enrichment;

use clap::Parser;
use crate::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logger if env vars are set
    env_logger::init();
    
    let args = Cli::parse();

    match &args.command {
        Some(Commands::Commit) => {
            let db = db::init_db("state.db")?;
            phases::commit(&db)?;
        }
        Some(Commands::Rollback) => {
            let db = db::init_db("state.db")?;
            phases::rollback(&db)?;
        }
        Some(Commands::Purge { force }) => {
            let db = db::init_db("state.db")?;
            phases::purge(&db, *force)?;
        }
        None => {
            if args.source.is_empty() || args.destination.is_none() {
                anyhow::bail!("Source and destination flags are required for scanning.");
            }
            println!("Starting file organization...");
            
            let db_path = "state.db";
            if !args.resume && std::path::Path::new(db_path).exists() {
                std::fs::remove_file(db_path)?;
            }
            let db = db::init_db(db_path)?;
            println!("Database initialized at {}.", db_path);
            
            phases::run_pipeline(&args, &db).await?;
        }
    }

    Ok(())
}
