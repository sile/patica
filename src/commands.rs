use crate::journal::JournalHttpServer;
use clap::{Args, Subcommand};
use pagurus::{
    event::{Event, Key, KeyEvent},
    failure::OrFail,
    Game,
};
use pagurus_tui::TuiSystem;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum Command {
    New(NewCommand),
    Open(OpenCommand),
}

impl Command {
    pub fn run(&self) -> pagurus::Result<()> {
        match self {
            Command::New(cmd) => cmd.run().or_fail(),
            Command::Open(cmd) => cmd.run().or_fail(),
        }
    }
}

#[derive(Debug, Args)]
pub struct NewCommand {
    pub name: PathBuf,
}

impl NewCommand {
    pub fn run(&self) -> pagurus::Result<()> {
        JournalHttpServer::start(&self.name, true).or_fail()?;
        println!("Created: {}", self.name.display());
        Ok(())
    }
}

// TODO: EditCommand
#[derive(Debug, Args)]
pub struct OpenCommand {
    pub name: PathBuf,
}

impl OpenCommand {
    pub fn run(&self) -> pagurus::Result<()> {
        JournalHttpServer::start(&self.name, false).or_fail()?;

        let mut system = TuiSystem::new().or_fail()?;
        let mut game = crate::game::Game::default();
        game.initialize(&mut system).or_fail()?;
        while let Ok(event) = system.next_event() {
            if is_quit_key(&event) {
                break;
            }
            if !game.handle_event(&mut system, event).or_fail()? {
                break;
            }
        }
        Ok(())
    }
}

fn is_quit_key(event: &Event) -> bool {
    let Event::Key(KeyEvent { key, ctrl,.. }) = event else {
        return false;
    };
    matches!(
        (key, ctrl),
        (Key::Esc, _) | (Key::Char('c'), true) | (Key::Char('q'), false)
    )
}
