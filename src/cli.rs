use crate::journal::JournaledModel;
use pagurus::{
    event::{Event, Key, KeyEvent},
    failure::OrFail,
    Game,
};
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
    SelectColor(SelectColorCommand),
}

impl Command {
    fn run(&self, path: &PathBuf) -> pagurus::Result<()> {
        match self {
            Command::Open(cmd) => cmd.run(path).or_fail(),
            Command::SelectColor(cmd) => cmd.run(path).or_fail(),
        }
    }
}

#[derive(Debug, clap::Args)]
struct OpenCommand;

impl OpenCommand {
    fn run(&self, path: &PathBuf) -> pagurus::Result<()> {
        let mut journal = JournaledModel::open_or_create(path).or_fail()?;

        let mut system = TuiSystem::new().or_fail()?;
        let mut game = crate::game::Game::default();
        game.initialize(&mut system).or_fail()?;

        while let Ok(event) = system.next_event() {
            if self.is_quit_key(&event) {
                break;
            }

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

    fn is_quit_key(&self, event: &Event) -> bool {
        let Event::Key(KeyEvent { key, ctrl, .. }) = event else {
            return false;
        };
        matches!(
            (key, ctrl),
            (Key::Esc, _) | (Key::Char('c'), true) | (Key::Char('q'), false)
        )
    }
}

#[derive(Debug, clap::Args)]
struct SelectColorCommand {
    color_index: usize,
}

impl SelectColorCommand {
    fn run(&self, path: &PathBuf) -> pagurus::Result<()> {
        JournaledModel::open_if_exists(path)
            .or_fail()?
            .with_locked_model(|model| model.select_color(self.color_index).or_fail())
            .or_fail()?;
        Ok(())
    }
}
