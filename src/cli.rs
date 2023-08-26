use pagurus::{failure::OrFail, Game};
use pagurus_tui::{TuiSystem, TuiSystemOptions};
use pati::{CommandReader, CommandWriter, Point, VersionedCanvas};
use std::{
    collections::BTreeMap,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use crate::{
    command::Command,
    config::Config,
    frame::Frame,
    model::Model,
    remote::{RemoteCommandClient, RemoteCommandServer},
};

#[derive(Debug, clap::Parser)]
#[clap(version, about)]
pub enum Args {
    Open(OpenCommand),
    Apply(ApplyCommand),
    Include(IncludeCommand),
    Embed(EmbedCommand),
    Export(ExportCommand),
}

impl Args {
    pub fn run(&self) -> orfail::Result<()> {
        match self {
            Self::Open(cmd) => cmd.run().or_fail(),
            Self::Apply(cmd) => cmd.run().or_fail(),
            Self::Include(cmd) => cmd.run().or_fail(),
            Self::Embed(cmd) => cmd.run().or_fail(),
            Self::Export(cmd) => cmd.run().or_fail(),
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

        let mut embedded_canvases = BTreeMap::new();
        for embedded in game.model().frames().values() {
            let canvas = EmbeddedCanvas::new(&embedded.frame.path).or_fail()?;
            embedded_canvases.insert(embedded.frame.path.clone(), canvas);
        }
        for embedded_canvas in embedded_canvases.values_mut() {
            embedded_canvas.sync(game.model_mut()).or_fail()?;
        }

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

            // TODO: optimize
            for embedded in game.model().frames().values() {
                if embedded_canvases.contains_key(&embedded.frame.path) {
                    continue;
                }
                let canvas = EmbeddedCanvas::new(&embedded.frame.path).or_fail()?;
                embedded_canvases.insert(embedded.frame.path.clone(), canvas);
            }

            let mut removed = Vec::new();
            for embedded_canvas in embedded_canvases.values_mut() {
                if !embedded_canvas.sync(game.model_mut()).or_fail()? {
                    removed.push(embedded_canvas.path.clone());
                }
            }
            for path in removed {
                embedded_canvases.remove(&path);
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct EmbeddedCanvas {
    path: PathBuf,
    canvas: VersionedCanvas,
    reader: CommandReader<BufReader<std::fs::File>>,
}

impl EmbeddedCanvas {
    fn new(path: &PathBuf) -> orfail::Result<Self> {
        let file = std::fs::File::open(path).or_fail()?;
        let reader = CommandReader::new(BufReader::new(file));
        let canvas = VersionedCanvas::default();
        Ok(Self {
            path: path.clone(),
            canvas,
            reader,
        })
    }

    fn sync(&mut self, model: &mut Model) -> orfail::Result<bool> {
        while let Some(command) = self.reader.read_command().or_fail()? {
            self.canvas.apply(&command).or_fail()?;
        }
        for embedded in model.frames_mut().values_mut() {
            if embedded.frame.path != self.path {
                continue;
            }
            if self.canvas.version() == embedded.version {
                continue;
            }
            embedded.sync(&self.canvas).or_fail()?;
        }
        Ok(model.frames().values().any(|f| f.frame.path == self.path))
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
        let canvas = load_canvas(&self.include_file, self.tag.as_ref()).or_fail()?;
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

#[derive(Debug, clap::Args)]
pub struct EmbedCommand {
    #[clap(short, long, default_value_t = 7539)]
    port: u16,

    #[clap(long = "start")]
    start_anchor: String,

    #[clap(long = "end")]
    end_anchor: String,

    #[clap(long)]
    name: String,

    path: PathBuf,
}

impl EmbedCommand {
    fn run(&self) -> orfail::Result<()> {
        let frame = Frame {
            name: self.name.clone(),
            path: self.path.clone(),
            start_anchor: self.start_anchor.clone(),
            end_anchor: self.end_anchor.clone(),
        };
        let command = Command::Embed(frame);
        apply_commands(self.port, &[command]).or_fail()?;
        Ok(())
    }
}

fn apply_commands(port: u16, commands: &[Command]) -> orfail::Result<()> {
    let mut client = RemoteCommandClient::connect(port).or_fail()?;
    client.send_commands(commands).or_fail()?;
    Ok(())
}

#[derive(Debug, clap::Args)]
pub struct ExportCommand {
    path: PathBuf,

    #[clap(short, long)]
    output: Option<PathBuf>,
}

impl ExportCommand {
    fn run(&self) -> orfail::Result<()> {
        let output = self
            .output
            .clone()
            .unwrap_or_else(|| self.path.with_extension("bmp"));
        let canvas = load_canvas(&self.path, None).or_fail()?;

        let mut start = Point::new(0, 0);
        let mut end = Point::new(0, 0);
        for point in canvas.pixels().keys().copied() {
            start.y = start.y.min(point.y);
            start.x = start.x.min(point.x);
            end.y = end.y.max(point.y);
            end.x = end.x.max(point.x);
        }
        crate::bmp::write_image(
            BufWriter::new(std::fs::File::create(&output).or_fail()?),
            (end.x - start.x + 1) as u16,
            (end.y - start.y + 1) as u16,
            canvas
                .pixels()
                .iter()
                .map(|(&point, &color)| (point - start, color)),
        )
        .or_fail()?;
        println!("Exported to {}", output.display());
        Ok(())
    }
}

fn load_canvas<P: AsRef<Path>>(path: &P, tag: Option<&String>) -> orfail::Result<pati::Canvas> {
    let file = std::fs::File::open(path).or_fail()?;
    let mut reader = CommandReader::new(BufReader::new(file));
    let mut canvas = pati::Canvas::new();
    let mut tagged_canvas = None;
    while let Some(command) = reader.read_command().or_fail()? {
        if let Some(tag) = tag {
            if let pati::Command::Tag { name, .. } = &command {
                if name == tag {
                    tagged_canvas = Some(canvas.clone());
                }
            }
        }

        canvas.apply(&command);
    }
    if let Some(tag) = tag {
        Ok(tagged_canvas
            .take()
            .or_fail_with(|()| format!("No such tag: {tag}"))?)
    } else {
        Ok(canvas)
    }
}
