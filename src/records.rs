use pagurus::failure::OrFail;
use serde::{Deserialize, Serialize};
use std::{io::Write, time::UNIX_EPOCH};

pub fn append_record<W: Write, T: Serialize>(mut writer: W, record: &T) -> pagurus::Result<()> {
    serde_json::to_writer(&mut writer, record).or_fail()?;
    writeln!(&mut writer).or_fail()?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Record {
    Create(CreateRecord),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct UnixTimestamp(u64);

impl UnixTimestamp {
    pub fn now() -> pagurus::Result<Self> {
        Ok(Self(UNIX_EPOCH.elapsed().or_fail()?.as_secs()))
    }
}
