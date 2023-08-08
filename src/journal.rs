use crate::{
    model::ModelCommand,
    records::{OpenRecord, Record},
};
use pagurus::failure::OrFail;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

#[derive(Debug)]
pub struct JournalHttpServer {
    writer: JournalWriter,
    socket: std::net::TcpListener,
    external_comamnds: Vec<ModelCommand>,
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
                _ => {}
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
            external_comamnds: Vec::new(),
        })
    }

    pub fn append_commands(&mut self, commands: Vec<ModelCommand>) -> pagurus::Result<()> {
        for command in commands {
            self.writer.append(&Record::Model(command)).or_fail()?;
        }
        Ok(())
    }

    pub fn take_external_commands(&mut self) -> Vec<ModelCommand> {
        std::mem::take(&mut self.external_comamnds)
    }
}

#[derive(Debug)]
pub struct JournalHttpClient {}

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
