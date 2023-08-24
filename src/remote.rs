use crate::command::Command;
use orfail::OrFail;
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, TryRecvError},
};

#[derive(Debug)]
pub struct RemoteCommandServer {
    tx: mpsc::Sender<Command>,
    listener: TcpListener,
    clients: Vec<CommandReader>,
}

impl RemoteCommandServer {
    pub fn start(port: u16) -> orfail::Result<RemoteCommandServerHandle> {
        let listener = TcpListener::bind(("127.0.0.1", port)).or_fail()?;
        listener.set_nonblocking(true).or_fail()?;

        let (tx, rx) = mpsc::channel();
        let server = Self {
            tx,
            listener,
            clients: Vec::new(),
        };
        std::thread::spawn(move || {
            server.run();
        });
        let handle = RemoteCommandServerHandle { rx };
        Ok(handle)
    }

    fn run(mut self) {
        loop {
            if let Err(e) = self.run_one().or_fail() {
                pagurus::println!("Error: {:?}", e);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn run_one(&mut self) -> orfail::Result<()> {
        match self.listener.accept() {
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    Err(e).or_fail()?;
                }
            }
            Ok((stream, _)) => {
                stream.set_nonblocking(true).or_fail()?;
                self.clients.push(CommandReader::new(stream));
            }
        }
        let mut i = 0;
        while i < self.clients.len() {
            match self.clients[i].read_command() {
                Err(_) => {
                    self.clients.swap_remove(i);
                }
                Ok(None) => {
                    i += 1;
                }
                Ok(Some(command)) => {
                    self.tx.send(command).or_fail()?;
                    i += 1;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct RemoteCommandServerHandle {
    rx: mpsc::Receiver<Command>,
}

impl RemoteCommandServerHandle {
    pub fn poll_command(&self) -> orfail::Result<Option<Command>> {
        match self.rx.try_recv() {
            Err(TryRecvError::Disconnected) => {
                Err(orfail::Failure::new("RemoteCommandServer aborted"))
            }
            Err(TryRecvError::Empty) => Ok(None),
            Ok(command) => Ok(Some(command)),
        }
    }
}

#[derive(Debug)]
struct CommandReader {
    reader: BufReader<TcpStream>,
    line: String,
}

impl CommandReader {
    fn new(stream: TcpStream) -> Self {
        Self {
            reader: BufReader::new(stream),
            line: String::new(),
        }
    }

    fn read_command(&mut self) -> orfail::Result<Option<Command>> {
        match self.reader.read_line(&mut self.line) {
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    Ok(None)
                } else {
                    Err(e).or_fail()
                }
            }
            Ok(0) => Ok(None),
            Ok(_) => {
                if self.line.ends_with('\n') {
                    let command = serde_json::from_str(&self.line).or_fail()?;
                    self.line.clear();
                    Ok(Some(command))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct RemoteCommandClient {
    stream: BufWriter<TcpStream>,
}

impl RemoteCommandClient {
    pub fn connect(port: u16) -> orfail::Result<Self> {
        let stream = BufWriter::new(TcpStream::connect(("127.0.0.1", port)).or_fail()?);
        Ok(Self { stream })
    }

    pub fn send_commands(&mut self, commands: &[Command]) -> orfail::Result<()> {
        for command in commands {
            serde_json::to_writer(&mut self.stream, command).or_fail()?;
            self.stream.write_all(b"\n").or_fail()?;
        }
        self.stream.flush().or_fail()?;
        Ok(())
    }
}
