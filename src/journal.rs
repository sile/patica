use crate::{
    model::ModelCommand,
    records::{OpenRecord, Record},
};
use pagurus::failure::OrFail;
use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    net::SocketAddr,
    path::Path,
};

#[derive(Debug)]
pub struct JournalHttpServer {
    writer: JournalWriter,
    socket: std::net::TcpListener,
    connections: HashMap<SocketAddr, ServerSideConnection>,
    proposed_commands: VecDeque<ModelCommand>,
}

impl JournalHttpServer {
    pub fn start<P: AsRef<Path>>(path: P, create: bool) -> pagurus::Result<Self> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create_new(create)
            .open(path)
            .or_fail()?;

        let mut port = 0;
        let mut proposed_commands = VecDeque::new();
        for record in JournalRecords::new(BufReader::new(&mut file)) {
            let record = record.or_fail()?;
            match record {
                Record::Open(x) => {
                    port = x.port;
                }
                Record::Model(x) => {
                    proposed_commands.push_back(x);
                }
            }
        }

        // TODO: Use another port if the port is already in use by other dotedit programs.
        let socket = std::net::TcpListener::bind(("127.0.0.1", port)).or_fail()?;
        socket.set_nonblocking(true).or_fail()?;

        let port = socket.local_addr().or_fail()?.port();
        let record = Record::Open(OpenRecord::new(port).or_fail()?);
        let mut writer = JournalWriter::new(file);
        writer.append(&record).or_fail()?;

        Ok(Self {
            writer,
            socket,
            connections: HashMap::new(),
            proposed_commands,
        })
    }

    pub fn handle_http_request(&mut self) -> pagurus::Result<()> {
        // match self.socket.accept() {
        //     Ok((mut stream, _)) => {
        //         todo!()
        //     }
        //     Err(e) => {
        //         return Err(e).or_fail()
        //     },
        // }
        Ok(())
    }

    pub fn append_commands(&mut self, commands: Vec<ModelCommand>) -> pagurus::Result<()> {
        for command in commands {
            self.writer.append(&Record::Model(command)).or_fail()?;
        }
        Ok(())
    }

    pub fn with_next_proposed_command<F>(&mut self, mut f: F) -> pagurus::Result<bool>
    where
        F: FnMut(ModelCommand) -> pagurus::Result<()>,
    {
        if let Some(command) = self.proposed_commands.pop_front() {
            // TODO: Handle error.
            f(command).or_fail()?;
        }
        Ok(!self.proposed_commands.is_empty())
    }
}

#[derive(Debug)]
struct ServerSideConnection {}

#[derive(Debug)]
pub struct JournalHttpClient {
    socket: std::net::TcpStream,
}

impl JournalHttpClient {
    pub fn connect(port: u16) -> pagurus::Result<Self> {
        let socket = std::net::TcpStream::connect(("127.0.0.1", port)).or_fail()?;
        Ok(Self { socket })
    }
}

#[derive(Debug)]
struct JournalWriter {
    writer: BufWriter<File>,
}

impl JournalWriter {
    fn new(file: File) -> Self {
        Self {
            writer: BufWriter::new(file),
        }
    }

    fn append(&mut self, record: &Record) -> pagurus::Result<()> {
        serde_json::to_writer(&mut self.writer, record).or_fail()?;
        writeln!(self.writer).or_fail()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct JournalRecords<R> {
    lines: std::io::Lines<R>,
}

impl<R: BufRead> JournalRecords<R> {
    pub fn new(reader: R) -> Self {
        Self {
            lines: reader.lines(),
        }
    }
}

impl<R: BufRead> Iterator for JournalRecords<R> {
    type Item = pagurus::Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next()?;
        Some(
            line.or_fail()
                .and_then(|line| serde_json::from_str(&line).or_fail()),
        )
    }
}
