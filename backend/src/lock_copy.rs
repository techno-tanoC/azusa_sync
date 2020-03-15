use std::borrow::Cow;
use std::fs::File;
use std::io::{self, Read, Write, BufWriter, Seek, SeekFrom};
use std::path::*;
use std::sync::Mutex;

use super::error::Result;

pub struct LockCopy {
    mutex: Mutex<()>,
    path: PathBuf,
}

impl LockCopy {
    pub fn new<P: AsRef<Path>>(path: &P) -> Self {
        LockCopy {
            mutex: Mutex::new(()),
            path: path.as_ref().to_path_buf()
        }
    }

    pub fn copy<F>(&self, from: &mut F, name: impl AsRef<str>, ext: impl AsRef<str>) -> Result<()>
        where
            F: Read + Seek
    {
        let _lock = self.mutex.lock();
        let fresh = Self::fresh_name(&self.path, &name, &ext);
        let mut to = BufWriter::new(File::create(&fresh)?);
        Self::rewind_and_copy(from, &mut to)?;
        Ok(())
    }

    fn rewind_and_copy<F, G>(from: &mut F, to: &mut G) -> Result<()>
        where
            F: Read + Seek,
            G: Write
    {
        from.seek(SeekFrom::Start(0))?;
        io::copy(from, to)?;
        Ok(())
    }

    fn fresh_name<P: AsRef<Path>>(path: &P, name: impl AsRef<str>, ext: impl AsRef<str>) -> PathBuf {
        let mut i = 0;
        loop {
            let name = Self::build_name(&name, i, &ext);
            let candidate = path.as_ref().join(name);
            if candidate.exists() {
                i += 1;
            } else {
                return candidate.to_path_buf();
            }
        }
    }

    fn build_name(name: impl AsRef<str>, count: u64, ext: impl AsRef<str>) -> String {
        let count: Cow<'_, _> = if count >= 1 {
            format!("({})", count).into()
        } else {
            "".into()
        };

        // todo redundant
        let ext: Cow<'_, _> = if ext.as_ref().is_empty() {
            "".into()
        } else {
            format!(".{}", ext.as_ref()).into()
        };

        name.as_ref().to_string() + &count + &ext
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // todo
    #[test]
    fn rewind_and_copy_test() {
    }

    #[test]
    fn fresh_name_test() {
        let fresh = LockCopy::fresh_name(&".", "dummy", "toml");
        assert_eq!(fresh.to_string_lossy(), "./dummy.toml".to_string());

        let fresh = LockCopy::fresh_name(&".", "Cargo", "toml");
        assert_eq!(fresh.to_string_lossy(), "./Cargo(1).toml".to_string());
    }

    #[test]
    fn build_name_test() {
        let name = LockCopy::build_name("hello", 0, "jpg");
        assert_eq!(name, "hello.jpg");

        let name = LockCopy::build_name("hello", 1, "jpg");
        assert_eq!(name, "hello(1).jpg");

        let name = LockCopy::build_name("hello", 0, "");
        assert_eq!(name, "hello");

        let name = LockCopy::build_name("hello", 1, "");
        assert_eq!(name, "hello(1)");
    }
}
