use pagurus::{failure::OrFail, Game};
use pagurus_tui::{TuiSystem, TuiSystemOptions};
use pati::{CommandReader, CommandWriter, Point};
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
    Include(IncludeCommand),
    // TODO: Export(ExportCommand),
    // Embed
}

impl Args {
    pub fn run(&self) -> orfail::Result<()> {
        match self {
            Self::Open(cmd) => cmd.run().or_fail(),
            Self::Apply(cmd) => cmd.run().or_fail(),
            Self::Include(cmd) => cmd.run().or_fail(),
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
    fn run(&self) -> orfail::Result<()> {
        let commands: Vec<Command> =
            serde_json::from_reader(&mut std::io::stdin().lock()).or_fail()?;
        apply_commands(self.port, &commands).or_fail()?;
        Ok(())
    }
}

#[derive(Debug, clap::Args)]
pub struct IncludeCommand {
    #[clap(short, long, default_value_t = 7539)]
    port: u16,

    #[clap(long)]
    start_anchor: Option<String>,

    #[clap(long)]
    end_anchor: Option<String>,

    #[clap(long)]
    tag: Option<String>,

    include_file: PathBuf,
}

impl IncludeCommand {
    fn run(&self) -> orfail::Result<()> {
        let file = std::fs::File::open(&self.include_file).or_fail()?;
        let mut reader = CommandReader::new(BufReader::new(file));
        let mut canvas = pati::Canvas::new();
        let mut tagged_canvas = None;
        while let Some(command) = reader.read_command().or_fail()? {
            if let Some(tag) = &self.tag {
                if let pati::Command::Tag { name, .. } = &command {
                    if name == tag {
                        tagged_canvas = Some(canvas.clone());
                    }
                }
            }

            canvas.apply(&command);
        }
        if let Some(tag) = &self.tag {
            tagged_canvas
                .is_some()
                .or_fail_with(|()| format!("No such tag: {tag}"))?;
        }

        if let Some(c) = tagged_canvas {
            canvas = c;
        }

        let mut start = Point::new(i16::MIN, i16::MIN);
        let mut end = Point::new(i16::MAX, i16::MAX);
        if let Some(anchor) = &self.start_anchor {
            start = canvas
                .anchors()
                .get(anchor)
                .copied()
                .or_fail_with(|()| format!("No such anchor: {anchor}"))?;
        }
        if let Some(anchor) = &self.end_anchor {
            end = canvas
                .anchors()
                .get(anchor)
                .copied()
                .or_fail_with(|()| format!("No such anchor: {anchor}"))?;
        }
        (start.x <= end.x && start.y <= end.y).or_fail_with(|()| {
            format!(
                "Empty range: start=[{},{}]({}), end=[{},{}]({})",
                start.x,
                start.y,
                self.start_anchor.as_ref().expect("unreachable"),
                end.x,
                end.y,
                self.end_anchor.as_ref().expect("unreachable"),
            )
        })?;
        let origin = Point::new(
            ((end.x as i32 - start.x as i32 + 1) / 2 + start.x as i32) as i16,
            ((end.y as i32 - start.y as i32 + 1) / 2 + start.y as i32) as i16,
        );

        let mut pixels = Vec::new();
        for (point, color) in canvas.range_pixels(start..=end) {
            pixels.push((point - origin, color));
        }
        let command = Command::Import(pixels);
        apply_commands(self.port, &[command]).or_fail()?;
        Ok(())
    }
}

fn apply_commands(port: u16, commands: &[Command]) -> orfail::Result<()> {
    let mut client = RemoteCommandClient::connect(port).or_fail()?;
    client.send_commands(commands).or_fail()?;
    Ok(())
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
//     fn run(&self) -> orfail::Result<()> {
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
