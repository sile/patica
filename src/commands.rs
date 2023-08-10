use crate::journal_file::JournaledModel;
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
    Open(OpenCommand),
    SelectColor(SelectColorCommand),
}

impl Command {
    pub fn run(&self) -> pagurus::Result<()> {
        match self {
            Command::Open(cmd) => cmd.run().or_fail(),
            Command::SelectColor(cmd) => cmd.run().or_fail(),
        }
    }
}

// TODO: EditCommand
#[derive(Debug, Args)]
pub struct OpenCommand {
    pub name: PathBuf,
}

impl OpenCommand {
    pub fn run(&self) -> pagurus::Result<()> {
        let mut journal = JournaledModel::open(&self.name).or_fail()?;

        let mut system = TuiSystem::new().or_fail()?;
        let mut game = crate::game::Game::default();
        game.initialize(&mut system).or_fail()?;

        while let Ok(event) = system.next_event() {
            if is_quit_key(&event) {
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

#[derive(Debug, Args)]
pub struct SelectColorCommand {
    pub name: PathBuf,
    pub color_index: usize,
}

impl SelectColorCommand {
    pub fn run(&self) -> pagurus::Result<()> {
        // TODO: optimize
        // let file = std::fs::File::open(&self.name).or_fail()?;
        // let mut port = 0;
        // let mut uuid = None;
        // let mut version = Default::default();
        // for record in JournalRecords::new(BufReader::new(file)) {
        //     let record = record.or_fail()?;
        //     if let Record::Model(x) = &record {
        //         version = x.version();
        //     }
        //     if let Record::Open(v) = record {
        //         port = v.port;
        //         uuid = Some(v.uuid);
        //     }
        // }

        // let mut client = JournalHttpClient::connect(port).or_fail()?;
        // let request = Request::Command {
        //     uuid: uuid.or_fail()?,
        //     command: ModelCommand::SelectColor {
        //         version: version.next(),
        //         index: ColorIndex(self.color_index),
        //     },
        // };
        // client.post(request).or_fail()?;

        Ok(())
    }
}
