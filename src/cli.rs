use crate::{game::Game, model::Model};
use orfail::OrFail;
use pagurus::Game as _;
use pagurus_tui::{TuiSystem, TuiSystemOptions};
use paticanvas::{CanvasAgentRequest, CanvasAgentServer, CanvasFile};
use std::path::PathBuf;

const ENV_PATICA_PORT: &str = "PATICA_PORT";

// use crate::{
//     clock::Ticks,
//     command::{AnchorName, CenterPoint, Command, MoveDestination},
//     config::Config,
//     frame::Frame,
//     model::Model,
//     remote::{RemoteCommandClient, RemoteCommandServer},
// };
// use pagurus::{failure::OrFail, Game};
// use pagurus_tui::{TuiSystem, TuiSystemOptions};
// use pati::{ImageCommandReader, ImageCommandWriter, Point, VersionedImage};
// use std::io::Write;
// use std::{
//     collections::BTreeMap,
//     io::{BufReader, BufWriter},
//     path::{Path, PathBuf},
// };

#[derive(Debug, clap::Parser)]
#[clap(version, about)]
pub enum Args {
    Open(OpenCommand),
    // Apply(ApplyCommand), // TODO: Rename to Command
    // Include(IncludeCommand),
    // Embed(EmbedCommand),
    // Export(ExportCommand),
    // #[clap(subcommand)]
    // Get(GetCommand), // TODO: Rename to Query
}

impl Args {
    pub fn run(&self) -> orfail::Result<()> {
        match self {
            Self::Open(cmd) => cmd.run().or_fail().map_err(|e| {
                // This is needed to leave the raw terminal mode before printing the error.
                println!();
                e
            }),
            // Self::Apply(cmd) => cmd.run().or_fail(),
            // Self::Include(cmd) => cmd.run().or_fail(),
            // Self::Embed(cmd) => cmd.run().or_fail(),
            // Self::Export(cmd) => cmd.run().or_fail(),
            // Self::Get(cmd) => cmd.run().or_fail(),
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct OpenCommand {
    path: PathBuf,
    // TODO: --config
}

impl OpenCommand {
    fn run(&self) -> orfail::Result<()> {
        let canvas_file = CanvasFile::open(&self.path, true).or_fail()?;
        let mut game = Game::new(Model::new(canvas_file));

        let mut agent_server = CanvasAgentServer::start().or_fail()?;
        std::env::set_var(ENV_PATICA_PORT, agent_server.port().to_string());

        let options = TuiSystemOptions {
            disable_mouse: true,
        };
        let mut system = TuiSystem::with_options(options).or_fail()?;
        game.initialize(&mut system).or_fail()?;

        while let Ok(event) = system.next_event() {
            if !game.handle_event(&mut system, event).or_fail()? {
                break;
            }
            self.poll_and_handle_request(&mut game, &mut agent_server)
                .or_fail()?;
        }

        Ok(())
    }

    fn poll_and_handle_request(
        &self,
        game: &mut Game,
        server: &mut CanvasAgentServer,
    ) -> orfail::Result<()> {
        let Some((from, request)) = server.poll_request().or_fail()? else {
            return Ok(());
        };
        match request {
            CanvasAgentRequest::Command(command) => {
                game.model_mut().command(&command).or_fail()?;
                server.send_response(from, ()).or_fail()?;
            }
            CanvasAgentRequest::Query(query) => {
                let value = game.model().query(&query);
                server.send_response(from, value).or_fail()?;
            }
        }
        Ok(())
    }
}

// #[derive(Debug)]
// struct EmbeddedCanvas {
//     path: PathBuf,
//     canvas: VersionedImage,
//     reader: ImageCommandReader<BufReader<std::fs::File>>,
// }

// impl EmbeddedCanvas {
//     fn new(path: &PathBuf) -> orfail::Result<Self> {
//         let file = std::fs::File::open(path).or_fail()?;
//         let reader = ImageCommandReader::new(BufReader::new(file));
//         let canvas = VersionedImage::default(); // TODO: use Canvas
//         Ok(Self {
//             path: path.clone(),
//             canvas,
//             reader,
//         })
//     }

//     fn sync(&mut self, model: &mut Model) -> orfail::Result<bool> {
//         while let Some(command) = self.reader.read_command().or_fail()? {
//             self.canvas.apply(&command).or_fail()?;
//         }
//         for embedded in model.frames_mut().values_mut() {
//             if embedded.frame.path != self.path {
//                 continue;
//             }
//             if self.canvas.version() == embedded.version {
//                 continue;
//             }
//             embedded.sync(&self.canvas).or_fail()?;
//         }
//         Ok(model.frames().values().any(|f| f.frame.path == self.path))
//     }
// }

// #[derive(Debug, clap::Args)]
// pub struct ApplyCommand {
//     #[clap(short, long, default_value_t = 7539)]
//     port: u16,
//     // TODO: path
// }

// impl ApplyCommand {
//     fn run(&self) -> orfail::Result<()> {
//         let commands: Vec<Command> =
//             serde_json::from_reader(&mut std::io::stdin().lock()).or_fail()?;
//         apply_commands(self.port, &commands).or_fail()?;
//         Ok(())
//     }
// }

// #[derive(Debug, clap::Args)]
// pub struct IncludeCommand {
//     #[clap(short, long, default_value_t = 7539)]
//     port: u16,

//     #[clap(long)]
//     start_anchor: Option<String>,

//     #[clap(long)]
//     end_anchor: Option<String>,

//     include_file: PathBuf,
// }

// impl IncludeCommand {
//     fn run(&self) -> orfail::Result<()> {
//         let canvas = load_canvas(&self.include_file).or_fail()?;
//         let mut start = Point::new(i16::MIN, i16::MIN);
//         let mut end = Point::new(i16::MAX, i16::MAX);
//         if let Some(anchor) = &self.start_anchor {
//             start = canvas
//                 .anchors()
//                 .get(anchor)
//                 .copied()
//                 .or_fail_with(|()| format!("No such anchor: {anchor}"))?;
//         }
//         if let Some(anchor) = &self.end_anchor {
//             end = canvas
//                 .anchors()
//                 .get(anchor)
//                 .copied()
//                 .or_fail_with(|()| format!("No such anchor: {anchor}"))?;
//         }
//         (start.x <= end.x && start.y <= end.y).or_fail_with(|()| {
//             format!(
//                 "Empty range: start=[{},{}]({}), end=[{},{}]({})",
//                 start.x,
//                 start.y,
//                 self.start_anchor.as_ref().expect("unreachable"),
//                 end.x,
//                 end.y,
//                 self.end_anchor.as_ref().expect("unreachable"),
//             )
//         })?;
//         let origin = Point::new(
//             ((end.x as i32 - start.x as i32 + 1) / 2 + start.x as i32) as i16,
//             ((end.y as i32 - start.y as i32 + 1) / 2 + start.y as i32) as i16,
//         );

//         let mut pixels = Vec::new();
//         for (point, color) in canvas.range_pixels(start..=end) {
//             pixels.push((point - origin, color));
//         }
//         let command = Command::Import(pixels);
//         apply_commands(self.port, &[command]).or_fail()?;
//         Ok(())
//     }
// }

// #[derive(Debug, clap::Args)]
// pub struct EmbedCommand {
//     #[clap(short, long, default_value_t = 7539)]
//     port: u16,

//     #[clap(long = "top-left")]
//     top_left_anchor: String,

//     #[clap(long = "bottom-right")]
//     bottom_right_anchor: String,

//     #[clap(long = "time", default_value_t = 0)]
//     start_ticks: u32,

//     #[clap(long = "duration", default_value_t = 1)]
//     duration_ticks: u32,

//     #[clap(long)]
//     name: String,

//     path: PathBuf,
//     // TODO: nosync
// }

// impl EmbedCommand {
//     fn run(&self) -> orfail::Result<()> {
//         let frame = Frame {
//             name: self.name.clone(),
//             path: self.path.clone(),
//             top_left_anchor: self.top_left_anchor.clone(),
//             bottom_right_anchor: self.bottom_right_anchor.clone(),
//             start_ticks: Ticks::new(self.start_ticks),
//             end_ticks: Ticks::new(self.start_ticks + self.duration_ticks),
//         };
//         let command = Command::Embed(frame);
//         apply_commands(self.port, &[command]).or_fail()?;
//         Ok(())
//     }
// }

// fn apply_commands(port: u16, commands: &[Command]) -> orfail::Result<()> {
//     let mut client = RemoteCommandClient::connect(port).or_fail()?;
//     client.send_commands(commands).or_fail()?;
//     Ok(())
// }

// #[derive(Debug, clap::Args)]
// pub struct ExportCommand {
//     path: PathBuf,

//     #[clap(short, long)]
//     output: Option<PathBuf>,
// }

// impl ExportCommand {
//     fn run(&self) -> orfail::Result<()> {
//         let output = self
//             .output
//             .clone()
//             .unwrap_or_else(|| self.path.with_extension("bmp"));
//         let canvas = load_canvas(&self.path).or_fail()?;

//         let mut start = Point::new(0, 0);
//         let mut end = Point::new(0, 0);
//         for point in canvas.pixels().keys().copied() {
//             start.y = start.y.min(point.y);
//             start.x = start.x.min(point.x);
//             end.y = end.y.max(point.y);
//             end.x = end.x.max(point.x);
//         }
//         crate::bmp::write_image(
//             BufWriter::new(std::fs::File::create(&output).or_fail()?),
//             (end.x - start.x + 1) as u16,
//             (end.y - start.y + 1) as u16,
//             canvas
//                 .pixels()
//                 .iter()
//                 .map(|(&point, &color)| (point - start, color)),
//         )
//         .or_fail()?;
//         println!("Exported to {}", output.display());
//         Ok(())
//     }
// }

// fn load_canvas<P: AsRef<Path>>(path: &P) -> orfail::Result<pati::Image> {
//     let file = std::fs::File::open(path).or_fail()?;
//     let mut reader = ImageCommandReader::new(BufReader::new(file));
//     let mut canvas = pati::Image::new();
//     while let Some(command) = reader.read_command().or_fail()? {
//         canvas.apply(&command);
//     }
//     Ok(canvas)
// }

// #[derive(Debug, clap::Subcommand)]
// pub enum GetCommand {
//     // TODO: port arg
//     BackgroundColor { path: PathBuf },
//     BrushColor { path: PathBuf },
//     Anchors { path: PathBuf },
// }

// impl GetCommand {
//     fn run(&self) -> orfail::Result<()> {
//         match self {
//             GetCommand::BackgroundColor { .. } => {
//                 let model = self.load_model().or_fail()?;
//                 self.output(model.background_color()).or_fail()?;
//             }
//             GetCommand::BrushColor { .. } => {
//                 let model = self.load_model().or_fail()?;
//                 self.output(model.brush_color()).or_fail()?;
//             }
//             GetCommand::Anchors { .. } => {
//                 let model = self.load_model().or_fail()?;
//                 self.output(model.canvas().anchors()).or_fail()?;
//             }
//         }
//         Ok(())
//     }

//     fn load_model(&self) -> orfail::Result<Model> {
//         let file = std::fs::File::open(self.path()).or_fail()?;
//         let mut reader = ImageCommandReader::new(BufReader::new(file));
//         let mut model = Model::default();
//         while let Some(command) = reader.read_command().or_fail()? {
//             model.canvas_mut().apply(&command);
//         }
//         model.initialize().or_fail()?;
//         Ok(model)
//     }

//     fn output(&self, value: impl serde::Serialize) -> orfail::Result<()> {
//         let stdout = std::io::stdout();
//         let mut stdout = stdout.lock();
//         serde_json::to_writer(&mut stdout, &value).or_fail()?;
//         writeln!(&mut stdout).or_fail()?;
//         Ok(())
//     }

//     fn path(&self) -> &PathBuf {
//         match self {
//             GetCommand::BackgroundColor { path } => path,
//             GetCommand::BrushColor { path } => path,
//             GetCommand::Anchors { path } => path,
//         }
//     }
// }
