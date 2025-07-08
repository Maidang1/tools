use clap::{Parser, Subcommand};
use std::process;

mod reminder;

#[derive(Parser)]
#[command(name = "tools-rs")]
#[command(about = "A collection of useful command-line tools")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Reminder management tool")]
    Reminder {
        #[command(subcommand)]
        action: reminder::ReminderAction,
    },
}

fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Reminder { action } => reminder::handle_reminder(action),
    };
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
