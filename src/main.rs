mod commands;
mod services;
mod types;

use clap::Parser;
use clap::Subcommand;
use commands::todo::TodoCommand;
use services::todo_service::TodoService;

#[derive(Subcommand, Debug)]
enum Commands {
    Todo {
        #[command(subcommand)]
        command: TodoCommand,
    },
}

#[derive(Parser)]
#[command(name = "milk-cap")]
#[command(about = "un tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Todo { command } => {
            let mut todo_service = TodoService::new();

            match command {
                TodoCommand::AddTodo { name } => todo_service.add_todo(name),
                TodoCommand::DeleteTodo { id } => todo_service.delete_todo(*id),
                TodoCommand::List => todo_service.list_todos(),
            }
        }
    }
}
