use std::fmt;
use std::io::{Read, Write, Seek, SeekFrom};
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};

pub struct ProgressInner<T> {
    pub(crate) name: String,
    pub(crate) total: u64,
    pub(crate) size: u64,
    pub(crate) canceled: bool,
    buf: T,
}

impl<T> ProgressInner<T> {
    fn new(name: impl ToString, buf: T) -> Self {
        ProgressInner {
            name: name.to_string(),
            total: 0,
            size: 0,
            canceled: false,
            buf,
        }
    }
}

impl<T> fmt::Debug for ProgressInner<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProgressInner")
            .field("name", &self.name)
            .field("total", &self.total)
            .field("size", &self.size)
            .field("canceled", &self.canceled)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct Progress<T>(Arc<Mutex<ProgressInner<T>>>);

impl<T> Progress<T> {
    pub fn new(name: impl ToString, buf: T) -> Self {
        Progress(Arc::new(Mutex::new(ProgressInner::new(name, buf))))
    }

    pub fn set_total(&self, total: u64) {
        self.lock().total = total;
    }

    pub fn cancel(&self) {
        self.lock().canceled = true;
    }

    pub(crate) fn lock(&self) -> MutexGuard<ProgressInner<T>> {
        self.0.lock().unwrap()
    }
}

impl<T: Seek> Progress<T> {
    pub fn rewind(&self) -> io::Result<u64> {
        self.lock().buf.seek(SeekFrom::Start(0))
    }
}

impl<T: Read> Read for Progress<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lock().buf.read(buf)
    }
}

impl<T: Write> Write for Progress<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut s = self.lock();
        if s.canceled {
            Err(io::Error::new(io::ErrorKind::Other, "canceled"))
        } else {
            let len = s.buf.write(buf)?;
            s.size += len as u64;
            Ok(len)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.lock().buf.flush()
    }
}

impl<T: Seek> Seek for Progress<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.lock().buf.seek(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    #[test]
    fn set_total_test() {
        let pg = Progress::new("pg", ());
        pg.set_total(1000);
        assert_eq!(pg.lock().total, 1000);
    }

    #[test]
    fn rewind_test() {
        let data = b"hello";
        let mut pg = Progress::new("pg", Cursor::new(vec![]));
        pg.write_all(&*data).unwrap();
        assert_eq!(pg.lock().buf.position(), 5);
        pg.rewind().unwrap();
        assert_eq!(pg.lock().buf.position(), 0);
    }

    #[test]
    fn read_test() {
        let data = b"hello";
        let mut pg = Progress::new("pg", Cursor::new(vec![]));
        pg.write_all(&*data).unwrap();
        pg.rewind().unwrap();

        let mut buf = vec![];
        pg.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, data);
    }

    #[test]
    fn write_test() {
        let data = b"hello";
        let mut pg = Progress::new("pg", Cursor::new(vec![]));
        pg.write_all(&*data).unwrap();

        assert_eq!(pg.lock().size, data.len() as u64);
        assert_eq!(pg.lock().buf.get_ref().clone(), data.to_vec());
    }
}
