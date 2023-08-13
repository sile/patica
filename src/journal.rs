use crate::model::{Command, Model};
use pagurus::failure::OrFail;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write},
    path::Path,
};

#[derive(Debug)]
pub struct JournaledModel {
    reader: BufReader<File>,
    writer: BufWriter<File>,
    model: Model,
    commands_len: usize,
}

impl JournaledModel {
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> pagurus::Result<Self> {
        Self::open(
            path,
            std::fs::OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .clone(),
        )
        .or_fail()
    }

    pub fn open_if_exists<P: AsRef<Path>>(path: P) -> pagurus::Result<Self> {
        Self::open(
            path,
            std::fs::OpenOptions::new().write(true).read(true).clone(),
        )
        .or_fail()
    }

    fn open<P: AsRef<Path>>(path: P, options: OpenOptions) -> pagurus::Result<Self> {
        let file = options.open(path.as_ref()).or_fail()?;
        let mut this = Self {
            reader: BufReader::new(file.try_clone().or_fail()?),
            writer: BufWriter::new(file),
            model: Model::default(),
            commands_len: 0,
        };
        this.sync_model().or_fail()?;
        Ok(this)
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut Model {
        &mut self.model
    }

    fn reload_if_need(&mut self) -> pagurus::Result<()> {
        if self.reader.get_ref().metadata().or_fail()?.len()
            < self.reader.stream_position().or_fail()?
        {
            self.model = Model::default();
            self.commands_len = 0;
            self.reader.seek(SeekFrom::Start(0)).or_fail()?;
            pagurus::println!("Reloaded");
        }
        Ok(())
    }

    pub fn sync_model(&mut self) -> pagurus::Result<()> {
        self.model.take_applied_commands().is_empty().or_fail()?;

        self.reload_if_need().or_fail()?;

        while let Some(command) = self.next_command().or_fail()? {
            self.model.redo(&command).or_fail()?;
            self.commands_len += 1;
        }

        Ok(())
    }

    pub fn commands_len(&self) -> usize {
        self.commands_len
    }

    pub fn append_applied_commands(&mut self) -> pagurus::Result<()> {
        for command in self.model.take_applied_commands() {
            serde_json::to_writer(&mut self.writer, &command).or_fail()?;
            self.writer.write_all(b"\n").or_fail()?;
            self.commands_len += 1;
        }
        self.writer.flush().or_fail()?;
        Ok(())
    }

    fn next_command(&mut self) -> pagurus::Result<Option<Command>> {
        loop {
            let mut line = String::new();
            let n = self.reader.read_line(&mut line).or_fail()?;
            if n == 0 {
                return Ok(None);
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.ends_with('\n') {
                return serde_json::from_str(&line).or_fail().map(Some);
            } else {
                self.reader.seek_relative(-(n as i64)).or_fail()?;
                return Ok(None);
            }
        }
    }
}
