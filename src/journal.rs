use crate::model::{Model, ModelCommand};
use pagurus::failure::{Failure, OrFail};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct JournaledModel {
    lock_path: PathBuf,
    reader: BufReader<File>,
    writer: BufWriter<File>,
    model: Model,
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
        let lock_extension = if let Some(e) = path.as_ref().extension() {
            format!("{}.lock", e.to_str().or_fail()?)
        } else {
            "lock".to_owned()
        };
        let mut this = Self {
            reader: BufReader::new(file.try_clone().or_fail()?),
            writer: BufWriter::new(file),
            lock_path: path.as_ref().with_extension(lock_extension).to_path_buf(),
            model: Model::default(),
        };
        this.sync_model().or_fail()?;
        Ok(this)
    }

    fn sync_model(&mut self) -> pagurus::Result<usize> {
        self.model.take_applied_commands().is_empty().or_fail()?;

        let mut n = 0;
        while let Some(command) = self.next_command().or_fail()? {
            self.model.apply(command).or_fail()?;
            self.model.take_applied_commands();
            n += 1;
        }

        Ok(n)
    }

    pub fn with_locked_model<F, T>(&mut self, f: F) -> pagurus::Result<T>
    where
        F: FnOnce(&mut Model) -> pagurus::Result<T>,
    {
        self.lock().or_fail()?;

        let result = self
            .sync_model()
            .or_fail()
            .and_then(|_| f(&mut self.model).or_fail())
            .and_then(|value| {
                self.append_applied_commands().or_fail()?;
                Ok(value)
            });

        std::fs::remove_file(&self.lock_path).or_fail()?;

        result
    }

    fn lock(&mut self) -> pagurus::Result<()> {
        let now = Instant::now();
        while let Err(e) = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.lock_path)
        {
            pagurus::println!("Cannot acquire lock: {} ({})", e, self.lock_path.display());

            if now.elapsed() > Duration::from_secs(1) {
                return Err(Failure::new().message("Cannot acquire lock (timeout)"));
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    fn append_applied_commands(&mut self) -> pagurus::Result<()> {
        for command in self.model.take_applied_commands() {
            serde_json::to_writer(&mut self.writer, &command).or_fail()?;
            self.writer.write_all(b"\n").or_fail()?;
        }
        self.writer.flush().or_fail()?;
        Ok(())
    }

    fn next_command(&mut self) -> pagurus::Result<Option<ModelCommand>> {
        let mut line = String::new();
        let n = self.reader.read_line(&mut line).or_fail()?;
        if n == 0 {
            Ok(None)
        } else if line.ends_with('\n') {
            serde_json::from_str(&line).or_fail().map(Some)
        } else {
            self.reader.seek_relative(-(n as i64)).or_fail()?;
            Ok(None)
        }
    }
}
