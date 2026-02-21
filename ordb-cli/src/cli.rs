use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "File Organizer CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Source directories to scan
    #[arg(short, long, num_args = 1..)]
    pub source: Vec<PathBuf>,

    /// Destination directory for organized files
    #[arg(short, long)]
    pub destination: Option<PathBuf>,

    /// Batch size for AI inference
    #[arg(long, default_value_t = 64)]
    pub batch_size: usize,

    /// Confidence threshold for CLIP classification
    #[arg(long, default_value_t = 0.3)]
    pub confidence_threshold: f32,

    /// Dry run (do not copy/move files)
    #[arg(long)]
    pub dry_run: bool,

    /// Resume from existing state.db
    #[arg(long)]
    pub resume: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Commit the organization (move originals to trash)
    Commit,
    /// Rollback the organization (delete destination)
    Rollback,
    /// Purge the trash
    Purge {
        #[arg(long)]
        force: bool,
    },
}
