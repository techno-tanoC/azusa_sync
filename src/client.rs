use reqwest::{blocking, header};
use std::io::{Read, Write, Seek};

use super::progress::Progress;
use super::table::Table;
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

    pub fn start<W: Write + Read + Seek + Clone>(&self, table: &Table<String, Progress<W>>, url: &str, buf: W, name: impl ToString) -> Result<()> {
        let mut pg = Progress::new(name, buf);
        let _add = table.add(pg.clone());
        let mut res = self.0.get(url).send()?;
        Client::download(&mut pg, &mut res)?;
        // copy
        Ok(())
    }

    fn download<W: Write>(pg: &mut Progress<W>, res: &mut blocking::Response) -> Result<()> {
        if res.status().is_success() {
            if let Some(cl) = Self::content_length(res) {
                pg.set_total(cl);
            }

            // it can return canceled error.
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
