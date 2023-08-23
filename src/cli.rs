use pagurus::{failure::OrFail, Game};
use pagurus_tui::{TuiSystem, TuiSystemOptions};
use pati::{CommandReader, CommandWriter};
use std::{
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use crate::config::Config;

#[derive(Debug, clap::Parser)]
#[clap(version, about)]
pub enum Args {
    Open(OpenCommand),
    // TODO: Apply(ApplyCommand),
    // TODO: Export(ExportCommand),
    // Include (set-origin -> cut -> reset-origin)
    // Summary
    // ShowStatusLine
    // Compaction or GC
}

impl Args {
    pub fn run(&self) -> orfail::Result<()> {
        match self {
            Self::Open(cmd) => cmd.run().or_fail(),
            // Self::Apply(cmd) => cmd.run().or_fail(),
            // Self::Export(cmd) => cmd.run().or_fail(),
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct OpenCommand {
    path: PathBuf,
}

impl OpenCommand {
    fn run(&self) -> orfail::Result<()> {
        let config = Config::load_config_file().or_fail()?.unwrap_or_default();

        let mut game = crate::game::Game::default();
        game.set_config(config);

        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.path)
            .or_fail()?;
        let mut reader = CommandReader::new(BufReader::new(file.try_clone().or_fail()?));
        let mut writer = CommandWriter::new(BufWriter::new(file));
        while let Some(command) = reader.read_command().or_fail()? {
            game.model_mut().canvas_mut().apply(&command);
        }

        let options = TuiSystemOptions {
            disable_mouse: true,
        };
        let mut system = TuiSystem::with_options(options).or_fail()?;
        game.initialize(&mut system).or_fail()?;

        while let Ok(event) = system.next_event() {
            let version = game.model().canvas().version();
            if !game.handle_event(&mut system, event).or_fail()? {
                break;
            }
            for command in game.model().canvas().applied_commands(version) {
                writer.write_command(command).or_fail()?;
            }
        }

        Ok(())
    }
}

// #[derive(Debug, clap::Args)]
// pub struct ApplyCommand {
//     path: PathBuf,
// }

// impl ApplyCommand {
//     fn run(&self) -> pagurus::Result<()> {
//         let mut journal = JournaledModel::open_if_exists(&self.path).or_fail()?;
//         let mut commands = Vec::new();
//         let stdin = std::io::stdin();

//         for line in stdin.lock().lines() {
//             let line = line.or_fail()?;
//             commands.extend(
//                 serde_json::from_str::<CommandOrCommands>(&line)
//                     .or_fail()?
//                     .into_commands(),
//             );
//         }

//         for command in commands {
//             journal.model_mut().apply(command).or_fail()?;
//         }
//         journal.append_applied_commands().or_fail()?;

//         Ok(())
//     }
// }

// #[derive(Debug, clap::Args)]
// pub struct ExportCommand {
//     path: PathBuf,

//     #[clap(short, long)]
//     output: Option<PathBuf>,

//     #[clap(long, default_value = "1")]
//     scale: NonZeroUsize,
//     // TODO: size, origin, anchor, tag
// }

// impl ExportCommand {
//     fn run(&self) -> pagurus::Result<()> {
//         let journal = JournaledModel::open_if_exists(&self.path).or_fail()?;
//         let output = self
//             .output
//             .clone()
//             .unwrap_or_else(|| self.path.with_extension("bmp"));

//         crate::bmp::write_image(
//             BufWriter::new(std::fs::File::create(&output).or_fail()?),
//             journal.model().pixels_region(),
//             journal.model().pixels(),
//         )
//         .or_fail()?;

//         println!("Exported to {}", output.display());
//         Ok(())
//     }
// }
