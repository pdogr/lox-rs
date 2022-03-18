#![feature(exit_status_error)]
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

extern crate tempfile;
use tempfile::NamedTempFile;

mod command;
pub use command::CommandUnderTest;

mod macros;
pub use macros::*;

#[derive(Debug, Clone)]
pub struct TestWriter {
    inner: Rc<RefCell<Vec<u8>>>,
}

impl TestWriter {
    pub fn new() -> Self {
        TestWriter {
            inner: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.borrow_mut().flush()
    }
}

impl Default for TestWriter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn empty_tif() -> NamedTempFile {
    NamedTempFile::new().expect("Error: Cannot create input temp file.")
}

pub fn tif(input: String) -> NamedTempFile {
    let mut fio = NamedTempFile::new().expect("Error: Cannot create input temp file.");
    fio.write_all(input.as_bytes())
        .expect("Error: Cannot write to file.");
    fio
}

pub fn bench_cmd(mut cmd: CommandUnderTest, args: &[&str]) {
    let cmd = cmd.args(args);
    cmd.run()
        .exit_ok()
        .expect("Error: Command did not run successfully.")
}
