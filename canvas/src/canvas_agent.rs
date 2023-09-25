use crate::{
    command::CanvasCommand,
    query::{CanvasQuery, CanvasQueryValue},
};
use orfail::OrFail;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    net::{SocketAddr, TcpListener, TcpStream},
};
use std::{
    io::{Read, Write},
    time::Duration,
};

#[derive(Debug)]
pub struct CanvasAgentServer {
    listener: TcpListener,
    port: u16,
    clients: HashMap<SocketAddr, ClientState>,
}

impl CanvasAgentServer {
    pub fn start() -> orfail::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").or_fail()?;
        listener.set_nonblocking(true).or_fail()?;
        let port = listener.local_addr().or_fail()?.port();
        Ok(Self {
            listener,
            port,
            clients: HashMap::new(),
        })
    }

    pub fn poll_request(&mut self) -> orfail::Result<Option<(SocketAddr, CanvasAgentRequest)>> {
        match self.listener.accept() {
            Ok((stream, addr)) => {
                stream
                    .set_read_timeout(Some(Duration::from_secs(0)))
                    .or_fail()?;
                self.clients
                    .insert(addr, ClientState::new(stream).or_fail()?);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e).or_fail(),
        }

        let mut closed = vec![];
        for (&addr, client) in &mut self.clients {
            match client.poll() {
                Ok(Some(request)) => return Ok(Some((addr, request))),
                Ok(None) => {}
                Err(_) => closed.push(addr),
            }
        }
        for addr in closed {
            self.clients.remove(&addr);
        }
        Ok(None)
    }

    pub fn send_response(
        &mut self,
        addr: SocketAddr,
        response: impl Serialize,
    ) -> orfail::Result<()> {
        let client = self.clients.get_mut(&addr).or_fail()?;
        if send(&mut client.writer, response).is_err() {
            self.clients.remove(&addr);
        }
        Ok(())
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

#[derive(Debug)]
struct ClientState {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
    line: String,
}

impl ClientState {
    fn new(writer: TcpStream) -> orfail::Result<Self> {
        Ok(Self {
            reader: BufReader::new(writer.try_clone().or_fail()?),
            writer,
            line: String::new(),
        })
    }

    fn poll(&mut self) -> Result<Option<CanvasAgentRequest>, ()> {
        match self.reader.read_line(&mut self.line) {
            Ok(0) => Err(()),
            Ok(_) => {
                let request: CanvasAgentRequest =
                    serde_json::from_str(&self.line).map_err(|_| ())?;
                self.line.clear();
                Ok(Some(request))
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(_) => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct CanvasAgent {
    stream: TcpStream,
}

impl CanvasAgent {
    pub fn connect(port: u16) -> orfail::Result<Self> {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).or_fail()?;

        // Handshake
        send(&mut stream, Handshake::new()).or_fail()?;
        recv::<orfail::Result<()>>(&mut stream)
            .or_fail()?
            .or_fail()?;

        Ok(Self { stream })
    }

    pub fn command(&mut self, command: CanvasCommand) -> orfail::Result<()> {
        let request = CanvasAgentRequest::Command(command);
        send(&mut self.stream, &request).or_fail()?;
        recv(&mut self.stream).or_fail()
    }

    pub fn query(&mut self, query: CanvasQuery) -> orfail::Result<CanvasQueryValue> {
        let request = CanvasAgentRequest::Query(query);
        send(&mut self.stream, &request).or_fail()?;
        recv(&mut self.stream).or_fail()
    }
}

fn send(mut writer: impl Write, value: impl Serialize) -> orfail::Result<()> {
    serde_json::to_writer(&mut writer, &value).or_fail()?;
    writeln!(&mut writer).or_fail()?;
    writer.flush().or_fail()?;
    Ok(())
}

fn recv<T: for<'a> Deserialize<'a>>(mut reader: impl Read) -> orfail::Result<T> {
    let value: T = serde_json::from_reader(&mut reader).or_fail()?;
    Ok(value)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasAgentRequest {
    Command(CanvasCommand),
    Query(CanvasQuery),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct Handshake {
    magic_number: String,
}

impl Handshake {
    fn new() -> Self {
        Self {
            magic_number: "PATICA".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct HandshakeResult {}
