use pagurus::failure::OrFail;
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    time::UNIX_EPOCH,
};

#[derive(Debug)]
pub struct RecordWriter<W> {
    inner: W,
}

impl<W: Write> RecordWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    pub fn append(&mut self, record: &Record) -> pagurus::Result<()> {
        serde_json::to_writer(&mut self.inner, record).or_fail()?;
        writeln!(&mut self.inner).or_fail()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct RecordFile {
    file: BufReader<std::fs::File>,
}

impl RecordFile {
    pub fn open<P: AsRef<Path>>(path: P) -> pagurus::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(path)
            .or_fail()?;
        Ok(Self {
            file: BufReader::new(file),
        })
    }

    pub fn append(&mut self, record: &Record) -> pagurus::Result<()> {
        serde_json::to_writer(self.file.get_mut(), record).or_fail()?;
        writeln!(self.file.get_mut()).or_fail()?;
        Ok(())
    }

    pub fn next_record(&mut self) -> pagurus::Result<Option<Record>> {
        let mut line = String::new();
        let size = self.file.read_line(&mut line).or_fail()?;
        if size == 0 {
            return Ok(None);
        }
        serde_json::from_str(&line).or_fail().map(Some)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Record {
    Create(CreateRecord),
    Open(OpenRecord),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRecord {
    pub timestamp: UnixTimestamp,
    pub version: String,
}

impl CreateRecord {
    pub fn new() -> pagurus::Result<Self> {
        Ok(Self {
            timestamp: UnixTimestamp::now()?,
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenRecord {
    pub timestamp: UnixTimestamp,
    pub version: String,
    pub port: u16,
    // TODO: uuid
}

impl OpenRecord {
    pub fn new() -> pagurus::Result<Self> {
        Ok(Self {
            timestamp: UnixTimestamp::now()?,
            version: env!("CARGO_PKG_VERSION").to_string(),
            port: allocate_port().or_fail()?,
        })
    }

    pub fn with_port(port: u16) -> pagurus::Result<Self> {
        Ok(Self {
            timestamp: UnixTimestamp::now()?,
            version: env!("CARGO_PKG_VERSION").to_string(),
            port,
        })
    }
}

pub fn allocate_port() -> pagurus::Result<u16> {
    use std::net::TcpListener;
    let listener = TcpListener::bind(("127.0.0.1", 0)).or_fail()?;
    Ok(listener.local_addr().or_fail()?.port())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct UnixTimestamp(u64);

impl UnixTimestamp {
    pub fn now() -> pagurus::Result<Self> {
        Ok(Self(UNIX_EPOCH.elapsed().or_fail()?.as_secs()))
    }
}
