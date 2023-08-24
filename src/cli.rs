use pagurus::{failure::OrFail, Game};
use pagurus_tui::{TuiSystem, TuiSystemOptions};
use pati::{CommandReader, CommandWriter};
use std::{
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use crate::{
    command::Command,
    config::Config,
    remote::{RemoteCommandClient, RemoteCommandServer},
};

#[derive(Debug, clap::Parser)]
#[clap(version, about)]
pub enum Args {
    Open(OpenCommand),
    Apply(ApplyCommand),
    // TODO: Export(ExportCommand),
    // Include (set-origin -> cut -> reset-origin)
}

impl Args {
    pub fn run(&self) -> orfail::Result<()> {
        match self {
            Self::Open(cmd) => cmd.run().or_fail(),
            Self::Apply(cmd) => cmd.run().or_fail(),
            // Self::Export(cmd) => cmd.run().or_fail(),
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct OpenCommand {
    path: PathBuf,

    #[clap(short, long, default_value_t = 7539)]
    port: u16,
}

impl OpenCommand {
    fn run(&self) -> orfail::Result<()> {
        let server = RemoteCommandServer::start(self.port).or_fail()?;
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

            if let Some(patica_command) = server.poll_command().or_fail()? {
                game.model_mut().apply(&patica_command);
            }

            if !game.handle_event(&mut system, event).or_fail()? {
                break;
            }

            for pati_command in game.model().canvas().applied_commands(version) {
                writer.write_command(pati_command).or_fail()?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, clap::Args)]
pub struct ApplyCommand {
    #[clap(short, long, default_value_t = 7539)]
    port: u16,
}

impl ApplyCommand {
    fn run(&self) -> pagurus::Result<()> {
        let mut client = RemoteCommandClient::connect(self.port).or_fail()?;
        let commands: Vec<Command> =
            serde_json::from_reader(&mut std::io::stdin().lock()).or_fail()?;
        client.send_commands(&commands).or_fail()?;
        Ok(())
    }
}

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
