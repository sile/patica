use pagurus::failure::OrFail;
use serde::{Deserialize, Serialize};
use std::{io::Write, time::UNIX_EPOCH};

#[derive(Debug)]
pub struct RecordWriter<W> {
    inner: W,
}

impl<W: Write> RecordWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    pub fn append<T: Serialize>(&mut self, record: &T) -> pagurus::Result<()> {
        serde_json::to_writer(&mut self.inner, record).or_fail()?;
        writeln!(&mut self.inner).or_fail()?;
        Ok(())
    }
}

// - NAME
// - NAME.lock
// - NAME.tempXXX
// #[derive(Debug)]
// pub struct RecordReader<R> {
//     inner: R,
// }

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
}

impl OpenRecord {
    pub fn new(port: u16) -> pagurus::Result<Self> {
        Ok(Self {
            timestamp: UnixTimestamp::now()?,
            version: env!("CARGO_PKG_VERSION").to_string(),
            port,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct UnixTimestamp(u64);

impl UnixTimestamp {
    pub fn now() -> pagurus::Result<Self> {
        Ok(Self(UNIX_EPOCH.elapsed().or_fail()?.as_secs()))
    }
}
