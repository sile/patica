use crate::{command::CanvasCommand, Canvas};
use orfail::OrFail;
use pati::{ImageCommandReader, ImageCommandWriter, Version};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

#[derive(Debug)]
pub struct CanvasFile {
    canvas: Canvas,
    reader: ImageCommandReader<BufReader<File>>,
    writer: ImageCommandWriter<BufWriter<File>>,
    last_written_version: Version,
}

impl CanvasFile {
    pub fn open<P: AsRef<Path>>(path: P, create: bool) -> orfail::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(create)
            .open(&path)
            .or_fail_with(|e| format!("Failed to open file {}: {e}", path.as_ref().display()))?;
        let mut this = Self {
            canvas: Canvas::new(),
            reader: ImageCommandReader::new(BufReader::new(file.try_clone().or_fail()?)),
            writer: ImageCommandWriter::new(BufWriter::new(file)),
            last_written_version: Version::default(),
        };
        this.sync().or_fail()?;
        Ok(this)
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    pub fn sync(&mut self) -> orfail::Result<()> {
        while let Some(command) = self.reader.read_command().or_fail()? {
            self.canvas
                .command(&CanvasCommand::Image(command))
                .or_fail()?;
        }
        self.last_written_version = self.canvas.image().version();
        Ok(())
    }

    pub fn command(&mut self, command: &CanvasCommand) -> orfail::Result<()> {
        self.sync().or_fail()?;
        self.canvas.command(command).or_fail()?;
        for command in self
            .canvas
            .image()
            .applied_commands(self.last_written_version)
        {
            self.writer.write_command(command).or_fail()?;
        }
        self.last_written_version = self.canvas.image().version();
        Ok(())
    }
}
