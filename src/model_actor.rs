use crate::model::{Cursor, Model, ModelCommand, PixelPosition};
use pagurus::failure::OrFail;
use std::sync::mpsc;

#[derive(Debug)]
pub struct ModelActor {
    model: Model,
    request_rx: mpsc::Receiver<Request>,
}

impl ModelActor {
    pub fn new(model: Model) -> ModelActorHandle {
        let (request_tx, request_rx) = mpsc::channel();
        let actor = ModelActor { model, request_rx };
        std::thread::spawn(move || actor.run());
        ModelActorHandle { request_tx }
    }

    fn run(mut self) {
        loop {
            match self.run_once() {
                Ok(true) => {}
                Ok(false) => {
                    break;
                }
                Err(e) => {
                    pagurus::println!("ModelActor::run_once() failed: {}", e);
                    break;
                }
            }
        }
    }

    fn run_once(&mut self) -> pagurus::Result<bool> {
        if let Ok(request) = self.request_rx.recv() {
            match request {
                Request::Command(cmd) => {
                    self.model.handle_command(cmd).or_fail()?;
                    Ok(true)
                }
                Request::GetCursor(reply) => {
                    let _ = reply.send(self.model.cursor());
                    Ok(true)
                }
            }
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelActorHandle {
    request_tx: mpsc::Sender<Request>,
}

impl ModelActorHandle {
    pub fn move_cursor(&self, delta: PixelPosition) -> pagurus::Result<()> {
        self.command(ModelCommand::MoveCursor { delta }).or_fail()
    }

    pub fn get_cursor(&self) -> pagurus::Result<Cursor> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.request_tx
            .send(Request::GetCursor(reply_tx))
            .or_fail()?;
        reply_rx.recv().or_fail()
    }

    fn command(&self, cmd: ModelCommand) -> pagurus::Result<()> {
        self.request_tx.send(Request::Command(cmd)).or_fail()?;
        Ok(())
    }
}

impl Default for ModelActorHandle {
    fn default() -> Self {
        ModelActor::new(Model::default())
    }
}

#[derive(Debug)]
enum Request {
    Command(ModelCommand),
    GetCursor(mpsc::Sender<Cursor>),
}
