use crate::{
    config::Config,
    journal::JournaledModel,
    model::{Command, CommandOrCommands},
};
use pagurus::{failure::OrFail, Game};
use pagurus_tui::TuiSystem;
use std::{
    io::{BufRead, BufWriter},
    num::NonZeroUsize,
    path::PathBuf,
};

#[derive(Debug, clap::Parser)]
#[clap(version, about)]
pub enum Args {
    Open(OpenCommand),
    Apply(ApplyCommand),
    Export(ExportCommand),
    // Include (set-origin -> cut -> reset-origin)
    // Summary
    // ShowStatusLine
    // Compaction or GC
}

impl Args {
    pub fn run(&self) -> pagurus::Result<()> {
        match self {
            Self::Open(cmd) => cmd.run().or_fail(),
            Self::Apply(cmd) => cmd.run().or_fail(),
            Self::Export(cmd) => cmd.run().or_fail(),
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct ApplyCommand {
    path: PathBuf,
}

impl ApplyCommand {
    fn run(&self) -> pagurus::Result<()> {
        let mut journal = JournaledModel::open_if_exists(&self.path).or_fail()?;
        let mut commands = Vec::new();
        let stdin = std::io::stdin();

        for line in stdin.lock().lines() {
            let line = line.or_fail()?;
            commands.extend(
                serde_json::from_str::<CommandOrCommands>(&line)
                    .or_fail()?
                    .into_commands(),
            );
        }

        for command in commands {
            journal.model_mut().apply(command).or_fail()?;
        }
        journal.append_applied_commands().or_fail()?;

        Ok(())
    }
}

#[derive(Debug, clap::Args)]
pub struct OpenCommand {
    path: PathBuf,
}

impl OpenCommand {
    fn run(&self) -> pagurus::Result<()> {
        let config = Config::load_config_file().or_fail()?.unwrap_or_default();

        let mut journal = JournaledModel::open_or_create(&self.path).or_fail()?;
        if journal.commands_len() == 0 {
            journal
                .model_mut()
                .apply(Command::Header(Default::default()))
                .or_fail()?;
            for command in config.init.clone().into_commands() {
                journal.model_mut().apply(command).or_fail()?;
            }
            journal.append_applied_commands().or_fail()?;
        }

        let mut system = TuiSystem::new().or_fail()?;
        let mut game = crate::game::Game::default();

        game.set_config(config);
        game.initialize(&mut system).or_fail()?;

        while let Ok(event) = system.next_event() {
            journal.sync_model().or_fail()?;
            game.set_model(std::mem::take(journal.model_mut()));
            if !game.handle_event(&mut system, event).or_fail()? {
                break;
            }
            *journal.model_mut() = game.take_model().or_fail()?;
            journal.append_applied_commands().or_fail()?;
        }

        Ok(())
    }
}

#[derive(Debug, clap::Args)]
pub struct ExportCommand {
    path: PathBuf,

    #[clap(short, long)]
    output: Option<PathBuf>,

    #[clap(long, default_value = "1")]
    scale: NonZeroUsize,
    // TODO: size, origin, anchor, tag
}

impl ExportCommand {
    fn run(&self) -> pagurus::Result<()> {
        let journal = JournaledModel::open_if_exists(&self.path).or_fail()?;
        let output = self
            .output
            .clone()
            .unwrap_or_else(|| self.path.with_extension("bmp"));

        crate::bmp::write_image(
            BufWriter::new(std::fs::File::create(&output).or_fail()?),
            journal.model().pixels_region(),
            journal.model().pixels(),
        )
        .or_fail()?;

        println!("Exported to {}", output.display());
        Ok(())
    }
}
