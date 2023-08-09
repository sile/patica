use crate::{
    journal::{JournalHttpClient, JournalHttpServer, JournalRecords, Request},
    model::{ColorIndex, ModelCommand},
    records::Record,
};
use clap::{Args, Subcommand};
use pagurus::{
    event::{Event, Key, KeyEvent},
    failure::OrFail,
    Game,
};
use pagurus_tui::TuiSystem;
use std::{io::BufReader, path::PathBuf};

#[derive(Debug, Subcommand)]
pub enum Command {
    New(NewCommand),
    Open(OpenCommand),
    SelectColor(SelectColorCommand),
}

impl Command {
    pub fn run(&self) -> pagurus::Result<()> {
        match self {
            Command::New(cmd) => cmd.run().or_fail(),
            Command::Open(cmd) => cmd.run().or_fail(),
            Command::SelectColor(cmd) => cmd.run().or_fail(),
        }
    }
}

#[derive(Debug, Args)]
pub struct NewCommand {
    pub name: PathBuf,
}

impl NewCommand {
    pub fn run(&self) -> pagurus::Result<()> {
        JournalHttpServer::start(&self.name, true).or_fail()?;
        println!("Created: {}", self.name.display());
        Ok(())
    }
}

// TODO: EditCommand
#[derive(Debug, Args)]
pub struct OpenCommand {
    pub name: PathBuf,
}

impl OpenCommand {
    pub fn run(&self) -> pagurus::Result<()> {
        let mut journal = JournalHttpServer::start(&self.name, false).or_fail()?;

        let mut system = TuiSystem::new().or_fail()?;
        let mut game = crate::game::Game::default();
        game.initialize(&mut system).or_fail()?;
        while let Ok(event) = system.next_event() {
            // TODO: Move to other place
            {
                let mut updated = false;
                while journal
                    .with_next_proposed_command(|command| {
                        let data = serde_json::to_vec(&command).or_fail()?;
                        game.command(&mut system, "model.apply_command", &data)
                            .or_fail()?;
                        Ok(())
                    })
                    .or_fail()?
                {
                    updated = true;
                }
                if updated {
                    let _ = game
                        .query(&mut system, "model.take_applied_commands")
                        .or_fail()?;
                    system.request_redraw().or_fail()?;
                }
            }

            if is_quit_key(&event) {
                break;
            }
            if !game.handle_event(&mut system, event).or_fail()? {
                break;
            }

            let commands = serde_json::from_slice(
                &game
                    .query(&mut system, "model.take_applied_commands")
                    .or_fail()?,
            )
            .or_fail()?;
            journal.append_commands(commands).or_fail()?;

            journal
                .handle_http_request(&mut game, &mut system)
                .or_fail()?;
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
        let file = std::fs::File::open(&self.name).or_fail()?;
        let mut port = 0;
        let mut uuid = None;
        let mut version = Default::default();
        for record in JournalRecords::new(BufReader::new(file)) {
            let record = record.or_fail()?;
            if let Record::Model(x) = &record {
                version = x.version();
            }
            if let Record::Open(v) = record {
                port = v.port;
                uuid = Some(v.uuid);
            }
        }

        let mut client = JournalHttpClient::connect(port).or_fail()?;
        let request = Request::Command {
            uuid: uuid.or_fail()?,
            command: ModelCommand::SelectColor {
                version: version.next(),
                index: ColorIndex(self.color_index),
            },
        };
        client.post(request).or_fail()?;

        Ok(())
    }
}
