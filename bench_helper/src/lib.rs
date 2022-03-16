use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

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
