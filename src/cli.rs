use crate::{config::Config, journal::JournaledModel};
use pagurus::{failure::OrFail, Game};
use pagurus_tui::TuiSystem;
use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[clap(version, about)]
pub struct Args {
    file: PathBuf,

    #[clap(subcommand)]
    command: Command,
}

impl Args {
    pub fn run(&self) -> pagurus::Result<()> {
        self.command.run(&self.file).or_fail()
    }
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    Open(OpenCommand),
    #[clap(subcommand)]
    Set(SetCommand),
}

impl Command {
    fn run(&self, path: &PathBuf) -> pagurus::Result<()> {
        match self {
            Command::Open(cmd) => cmd.run(path).or_fail(),
            Command::Set(cmd) => cmd.run(path).or_fail(),
        }
    }
}

#[derive(Debug, clap::Args)]
struct OpenCommand;

impl OpenCommand {
    fn run(&self, path: &PathBuf) -> pagurus::Result<()> {
        let config = Config::load_config_file().or_fail()?.unwrap_or_default();

        let mut journal = JournaledModel::open_or_create(path).or_fail()?;
        if journal.applied_commands() == 0 {
            journal
                .with_locked_model(|model| {
                    for command in config.init.commands() {
                        model.apply(command.clone()).or_fail()?;
                    }
                    Ok(())
                })
                .or_fail()?;
        }

        let mut system = TuiSystem::new().or_fail()?;
        let mut game = crate::game::Game::default();

        game.set_config(config);
        game.initialize(&mut system).or_fail()?;

        while let Ok(event) = system.next_event() {
            let playing = journal.with_locked_model(|model| {
                game.set_model(std::mem::take(model));
                let playing = !game.handle_event(&mut system, event).or_fail()?;
                *model = game.take_model().or_fail()?;
                Ok(playing)
            })?;

            if playing {
                break;
            }
        }
        Ok(())
    }
}

#[derive(Debug, clap::Subcommand)]
enum SetCommand {
    Color(SetColorCommand),
}

impl SetCommand {
    fn run(&self, path: &PathBuf) -> pagurus::Result<()> {
        match self {
            SetCommand::Color(cmd) => cmd.run(path).or_fail(),
        }
    }
}

#[derive(Debug, clap::Args)]
struct SetColorCommand {
    name: String,
}

impl SetColorCommand {
    fn run(&self, path: &PathBuf) -> pagurus::Result<()> {
        JournaledModel::open_if_exists(path)
            .or_fail()?
            .with_locked_model(|model| model.set_color(self.name.clone().into()).or_fail())
            .or_fail()?;
        Ok(())
    }
}
