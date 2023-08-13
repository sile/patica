use crate::{
    config::Config,
    journal::JournaledModel,
    model::{CommandOrCommands, PixelRegion},
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
    Apply(ApplyCommand),
    Export(ExportCommand),
    // Include (set-origin -> cut -> reset-origin)
    // Summary
    // Compaction or GC
}

impl Command {
    fn run(&self, path: &PathBuf) -> pagurus::Result<()> {
        match self {
            Command::Open(cmd) => cmd.run(path).or_fail(),
            Command::Apply(cmd) => cmd.run(path).or_fail(),
            Command::Export(cmd) => cmd.run(path).or_fail(),
        }
    }
}

#[derive(Debug, clap::Args)]
struct ApplyCommand;

impl ApplyCommand {
    fn run(&self, path: &PathBuf) -> pagurus::Result<()> {
        let mut journal = JournaledModel::open_if_exists(path).or_fail()?;
        let mut commands = Vec::new();
        let stdin = std::io::stdin();

        for line in stdin.lock().lines() {
            let line = line.or_fail()?;
            commands.extend(
                serde_json::from_str::<CommandOrCommands>(&line)
                    .or_fail()?
                    .into_iter(),
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
struct OpenCommand;

impl OpenCommand {
    fn run(&self, path: &PathBuf) -> pagurus::Result<()> {
        let config = Config::load_config_file().or_fail()?.unwrap_or_default();

        let mut journal = JournaledModel::open_or_create(path).or_fail()?;
        if journal.commands_len() == 0 {
            for command in config.init.clone().into_iter() {
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
struct ExportCommand {
    #[clap(short, long)]
    output: Option<PathBuf>,

    #[clap(long, default_value = "1")]
    scale: NonZeroUsize,
    // TODO: size, origin, anchor, tag
}

impl ExportCommand {
    fn run(&self, path: &PathBuf) -> pagurus::Result<()> {
        let journal = JournaledModel::open_if_exists(path).or_fail()?;
        let output = self
            .output
            .clone()
            .unwrap_or_else(|| path.with_extension("bmp"));

        let mut empty = true;
        let mut min_x = i16::MAX;
        let mut min_y = i16::MAX;
        let mut max_x = i16::MIN;
        let mut max_y = i16::MIN;
        for (position, _color) in journal.model().pixels() {
            min_x = min_x.min(position.x);
            min_y = min_y.min(position.y);
            max_x = max_x.max(position.x);
            max_y = max_y.max(position.y);
            empty = false;
        }
        (!empty).or_fail()?;

        crate::bmp::write_image(
            BufWriter::new(std::fs::File::create(&output).or_fail()?),
            PixelRegion::from_corners(min_x, min_y, max_x, max_y),
            journal.model().pixels(),
        )
        .or_fail()?;

        println!("Exported to {}", output.display());
        Ok(())
    }
}
