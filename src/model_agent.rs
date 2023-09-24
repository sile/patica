use orfail::OrFail;
use serde::Serialize;
use std::collections::HashMap;
use std::io::{BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

#[derive(Debug)]
pub struct ModelAgentServer {
    listener: TcpListener,
    port: u16,
    clients: HashMap<SocketAddr, BufReader<TcpStream>>,
}

impl ModelAgentServer {
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

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn next_request(&mut self) -> orfail::Result<Option<ModelAgentRequest>> {
        match self.listener.accept() {
            Ok((stream, addr)) => {
                stream
                    .set_read_timeout(Some(Duration::from_secs(0)))
                    .or_fail()?;
                self.clients.insert(addr, BufReader::new(stream));
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    return Err(e).or_fail();
                }
            }
        }

        let mut closed = Vec::new();
        for (&addr, client) in &mut self.clients {
            closed.push(addr);
        }
        for addr in closed {
            self.clients.remove(&addr);
        }
    }
}

#[derive(Debug)]
pub struct ModelAgentClient {
    stream: TcpStream,
}

impl ModelAgentClient {
    fn open(port: u16) -> orfail::Result<Self> {
        let stream = TcpStream::connect(("127.0.0.1", port)).or_fail()?;
        Ok(Self { stream })
    }
}

#[derive(Debug)]
pub enum ModelAgentRequest<'a> {
    Command { client: &'a mut TcpStream },
    Query { client: &'a mut TcpStream },
}

impl<'a> ModelAgentRequest<'a> {
    pub fn reply(self, response: &impl Serialize) -> orfail::Result<()> {
        let mut client = match self {
            Self::Command { client } => client,
            Self::Query { client } => client,
        };
        serde_json::to_writer(&mut client, response).or_fail()?;
        writeln!(client).or_fail()?;
        client.flush().or_fail()?;
        Ok(())
    }
}
