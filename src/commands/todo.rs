use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum TodoCommand {
    AddTodo { name: String },
    DeleteTodo { id: u32 },
    List,
    Interactive,
}
