use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Command {
    New(NewCommand),
}

#[derive(Debug, Args)]
pub struct NewCommand {
    pub name: String,
}
