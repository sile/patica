use crate::{
    game::Game,
    model::ModelCommand,
    records::{OpenRecord, Record},
};
use pagurus::{failure::OrFail, Game as _};
use pagurus_tui::TuiSystem;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    net::TcpStream,
    path::Path,
};

#[derive(Debug)]
pub struct JournalHttpServer {
    writer: JournalWriter,
    socket: std::net::TcpListener,
    connections: Vec<ServerSideConnection>,
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
            connections: Vec::new(),
            proposed_commands,
        })
    }

    pub fn handle_http_request(
        &mut self,
        game: &mut Game,
        system: &mut TuiSystem,
    ) -> pagurus::Result<()> {
        match self.socket.accept() {
            Ok((stream, addr)) => {
                pagurus::dbg!(addr);
                stream.set_nonblocking(true).or_fail()?;
                self.connections.push(ServerSideConnection::new(stream));
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    return Err(e).or_fail();
                }
            }
        }

        let mut connections = Vec::new();
        for mut connection in std::mem::take(&mut self.connections) {
            connection.handle_io();
            if let Some(req) = connection.take_request() {
                pagurus::dbg!(&req);
                let res = self.handle_request(game, system, req).or_fail()?;
                connection.set_response(res);
                connection.handle_io();
            }
            if !connection.is_closed() {
                connections.push(connection);
            }
        }
        self.connections = connections;

        Ok(())
    }

    fn handle_request(
        &mut self,
        game: &mut Game,
        system: &mut TuiSystem,
        req: Request,
    ) -> pagurus::Result<Response> {
        match req {
            Request::Command { uuid, command } => {
                pagurus::dbg!(uuid);
                let _ = uuid; // TDOO: check
                let data = serde_json::to_vec(&command).or_fail()?;
                let result = game.command(system, "model.apply_command", &data).or_fail();
                pagurus::dbg!(&result);
                Ok(Response::new(result).or_fail()?)
            }
        }
    }

    pub fn append_commands(&mut self, commands: Vec<ModelCommand>) -> pagurus::Result<()> {
        let mut updated = false;
        for command in commands {
            self.writer.append(&Record::Model(command)).or_fail()?;
            updated = true;
        }
        if updated {
            self.writer.flush().or_fail()?;
        }
        Ok(())
    }

    // TODO: delete
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

#[derive(Debug, Clone, Copy)]
enum Method {
    Get,
    Post,
}

#[derive(Debug)]
struct ServerSideConnection {
    stream: TcpStream,
    request: Option<Request>,
    response: Option<Response>,
    closed: bool,
    buf: Vec<u8>,
    buf_start: usize,
    buf_end: usize,
    method: Option<Method>,
    content_length: usize,
    is_body: bool,
}

impl ServerSideConnection {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            request: None,
            response: None,
            closed: false,
            buf: vec![0; 4096],
            buf_start: 0,
            buf_end: 0,
            method: None,
            content_length: 0,
            is_body: false,
        }
    }

    fn handle_io(&mut self) {
        if self.request.is_none() {
            if let Err(e) = self.read_request().or_fail() {
                pagurus::println!("{:?}", e);
                self.closed = true;
                return;
            }
        }
    }

    fn read_request(&mut self) -> pagurus::Result<()> {
        pagurus::dbg!("read_request");
        let n = match self.stream.read(&mut self.buf[self.buf_end..]) {
            Ok(n) => n,
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    return Err(e).or_fail();
                } else {
                    return Ok(());
                }
            }
        };
        if n == 0 {
            self.closed = true;
            return Ok(());
        }
        self.buf_end += n;
        if self.method.is_none() {
            if self.buf[..self.buf_end].starts_with(b"GET / HTTP/1.1\r\n") {
                self.method = Some(Method::Get);
                self.buf_start = 15;
            } else if self.buf[..self.buf_end].starts_with(b"POST / HTTP/1.1\r\n") {
                self.method = Some(Method::Post);
                self.buf_start = 16;
            } else if self.buf[..self.buf_end]
                .iter()
                .find(|b| **b == b'\r')
                .is_some()
            {
                self.closed = true;
                return Ok(());
            } else {
                return Ok(());
            }
        }
        pagurus::dbg!(self.method);
        if !self.is_body {
            while let Some(line_end) = self.buf[self.buf_start..self.buf_end]
                .iter()
                .position(|b| *b == b'\r')
            {
                let i = b"\nContent-Length:".len().min(line_end);
                if self.buf[self.buf_start..][..i].eq_ignore_ascii_case(b"\nContent-Length:") {
                    self.content_length = std::str::from_utf8(
                        &self.buf[self.buf_start..][..line_end][b"\nContent-Length:".len()..],
                    )
                    .or_fail()?
                    .trim()
                    .parse::<usize>()
                    .or_fail()?;
                }
                self.buf_start += line_end + 1;
                if self.buf[self.buf_start..].starts_with(b"\n\r\n") {
                    self.is_body = true;
                    self.buf_start += 3;
                    self.buf.resize(self.buf_start + self.content_length, 0);
                    break;
                }
            }
            if !self.is_body {
                return Ok(());
            }
        }
        pagurus::dbg!(self.content_length);
        while self.buf_end - self.buf_start < self.content_length {
            let n = match self.stream.read(&mut self.buf[self.buf_end..]) {
                Ok(n) => n,
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        return Err(e).or_fail();
                    } else {
                        return Ok(());
                    }
                }
            };
            if n == 0 {
                self.closed = true;
                return Ok(());
            }
            self.buf_end += n;
        }

        self.request = Some(
            serde_json::from_slice(&self.buf[self.buf_start..][..self.content_length]).or_fail()?,
        );
        pagurus::dbg!(&self.request);

        self.buf_start += self.content_length;
        self.buf.drain(..self.buf_start);
        self.buf.resize(4096, 0);
        self.buf_end -= self.buf_start;
        self.buf_start = 0;
        self.method = None;
        self.is_body = false;
        self.content_length = 0;

        Ok(())
    }

    fn is_closed(&self) -> bool {
        self.closed
    }

    fn take_request(&mut self) -> Option<Request> {
        self.request.take()
    }

    fn set_response(&mut self, res: Response) {
        self.response = Some(res);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Command {
        uuid: uuid::Uuid,
        command: ModelCommand,
    },
}

// TODO: delete
#[derive(Debug, Serialize, Deserialize)]
struct Response(Vec<u8>);

impl Response {
    fn new<T: Serialize>(t: T) -> pagurus::Result<Self> {
        serde_json::to_vec(&t).or_fail().map(Self)
    }
}

#[derive(Debug)]
pub struct JournalHttpClient {
    socket: TcpStream,
}

impl JournalHttpClient {
    pub fn connect(port: u16) -> pagurus::Result<Self> {
        let socket = TcpStream::connect(("127.0.0.1", port)).or_fail()?;
        Ok(Self { socket })
    }

    pub fn post(&mut self, req: Request) -> pagurus::Result<()> {
        let body = serde_json::to_vec(&req).or_fail()?;
        self.socket.write_all(b"POST / HTTP/1.1\r\n").or_fail()?;
        self.socket
            .write_all(&format!("Content-Length: {}\r\n", body.len()).as_bytes())
            .or_fail()?;
        self.socket.write_all(b"\r\n").or_fail()?;
        self.socket.write_all(&body).or_fail()?;

        pagurus::dbg!("POST");
        // TODO: read response

        Ok(())
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

    fn flush(&mut self) -> pagurus::Result<()> {
        self.writer.flush().or_fail()?;
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
