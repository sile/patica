use crate::records::Record;
use pagurus::failure::{Failure, OrFail};
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct JournalFile {
    lock_path: PathBuf,
    reader: BufReader<File>,
    writer: BufWriter<File>,
}

impl JournalFile {
    pub fn create<P: AsRef<Path>>(path: P) -> pagurus::Result<()> {
        let _ = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create_new(true)
            .open(path)
            .or_fail()?;
        Ok(())
    }

    pub fn open<P: AsRef<Path>>(path: P) -> pagurus::Result<Self> {
        let file = File::open(path.as_ref()).or_fail()?;
        let lock_extension = if let Some(e) = path.as_ref().extension() {
            format!("{}.lock", e.to_str().or_fail()?)
        } else {
            "lock".to_owned()
        };
        Ok(Self {
            reader: BufReader::new(file.try_clone().or_fail()?),
            writer: BufWriter::new(file),
            lock_path: path.as_ref().with_extension(lock_extension).to_path_buf(),
        })
    }

    pub fn next_record(&mut self) -> pagurus::Result<Option<Record>> {
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

    pub fn lock(&mut self, timeout: Duration) -> pagurus::Result<(Vec<Record>, JournalFileLocked)> {
        let now = Instant::now();
        while std::fs::OpenOptions::new()
            .create_new(true)
            .open(&self.lock_path)
            .is_err()
        {
            if now.elapsed() > timeout {
                return Err(Failure::new().message("Cannot acquire lock (timeout)"));
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        let mut unread_records = Vec::new();
        while let Some(record) = self.next_record().or_fail()? {
            unread_records.push(record);
        }

        Ok((unread_records, JournalFileLocked::new(self)))
    }
}

#[derive(Debug)]
pub struct JournalFileLocked<'a> {
    inner: &'a mut JournalFile,
}

impl<'a> JournalFileLocked<'a> {
    fn new(inner: &'a mut JournalFile) -> Self {
        Self { inner }
    }

    pub fn append_record(&mut self, record: &Record) -> pagurus::Result<()> {
        serde_json::to_writer(&mut self.inner.writer, record).or_fail()?;
        self.inner.writer.write_all(b"\n").or_fail()?;
        self.inner.writer.flush().or_fail()?;
        Ok(())
    }
}

impl<'a> Drop for JournalFileLocked<'a> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.inner.lock_path);
    }
}

#[derive(Debug)]
pub struct JournalFileReadOnly(JournalFile);

impl JournalFileReadOnly {
    pub fn open<P: AsRef<Path>>(path: P) -> pagurus::Result<Self> {
        JournalFile::open(path).or_fail().map(Self)
    }

    pub fn next_record(&mut self) -> pagurus::Result<Option<Record>> {
        self.0.next_record().or_fail()
    }
}
