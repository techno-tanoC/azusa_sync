use reqwest::{blocking, header};
use std::io::{Read, Write, Seek};

use super::progress::Progress;
use super::table::Table;
use super::lock_copy::LockCopy;
use super::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct Client(reqwest::blocking::Client);

impl Client {
    pub fn new() -> Self {
        let client = blocking::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("ClientBuilder::build()");
        Client(client)
    }

    pub fn start<W>(&self, table: &Table<String, Progress<W>>, buf: W, url: &str, name: impl AsRef<str>, ext: impl AsRef<str>) -> Result<()>
        where W: Read + Write + Seek + Clone
    {
        let mut pg = Progress::new(name.as_ref(), buf);
        let _add = table.add(pg.clone());
        let mut res = self.0.get(url).send()?;
        Client::download(&mut res, &mut pg)?;
        // todo fix
        LockCopy::new(&".").copy(&mut pg, name.as_ref(), ext.as_ref())?;
        Ok(())
    }

    fn download<W: Write>(res: &mut blocking::Response, pg: &mut Progress<W>) -> Result<()> {
        if res.status().is_success() {
            if let Some(cl) = Self::content_length(res) {
                pg.set_total(cl);
            }

            // todo it can return canceled error.
            std::io::copy(res, pg)?;
            Ok(())
        } else {
            Err(Error::NonSuccessStatusError(format!("{:?}", res)))?
        }
    }

    fn content_length(res: &blocking::Response) -> Option<u64> {
        res.headers()
            .get(header::CONTENT_LENGTH)?
            .to_str().ok()?.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // todo
    #[test]
    fn content_length_test() {
    }
}