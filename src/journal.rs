use crate::records::{OpenRecord, Record};
use pagurus::failure::OrFail;
use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
};

fn write_record<W: Write>(mut writer: W, record: &Record) -> pagurus::Result<()> {
    serde_json::to_writer(&mut writer, record).or_fail()?;
    writeln!(writer).or_fail()?;
    Ok(())
}

#[derive(Debug)]
pub struct JournalHttpServer {
    file: std::fs::File,
    socket: std::net::TcpListener,
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
        for record in JournalRecords::new(BufReader::new(&mut file)) {
            let record = record.or_fail()?;
            match record {
                Record::Open(x) => {
                    port = x.port;
                }
            }
        }

        let socket = std::net::TcpListener::bind(("127.0.0.1", port)).or_fail()?;
        let port = socket.local_addr().or_fail()?.port();
        let record = Record::Open(OpenRecord::new(port).or_fail()?);
        write_record(&mut file, &record).or_fail()?;

        Ok(Self { file, socket })
    }
}

#[derive(Debug)]
pub struct JournalHttpClient {}

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
