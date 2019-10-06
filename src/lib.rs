#[macro_use] extern crate failure;

use async_std::prelude::*;
use futures::{TryFutureExt, Stream, StreamExt, TryStreamExt};
use futures::future::ready;
use async_std::io::BufReader;
use failure::Error;
use std::str::FromStr;
use std::convert::TryFrom;
use std::ops;

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

#[derive(Default, Clone, Debug)]
pub struct Locator(String);

impl Locator {
    pub fn root() -> Self {
        Self::default()
    }

    fn validate<T: AsRef<str>>(locator: T) -> Result<T> {
        let contains_line_ending = locator.as_ref().contains(&['\r', '\n'][..]);
        ensure!(!contains_line_ending, "Locator may not contain CR or LF");

        Ok(locator)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Locator {
    type Err = Error;

    fn from_str(locator: &str) -> Result<Self> {
        Self::validate(locator)?;

        Ok(Self(locator.into()))
    }
}

impl TryFrom<String> for Locator {
    type Error = Error;

    fn try_from(locator: String) -> Result<Self> {
        let locator = Self::validate(locator)?;

        Ok(Self(locator))
    }
}

impl ops::Deref for Locator {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

mod raw {
    use async_std::prelude::*;
    use async_std::io::prelude::*;
    use async_std::net::TcpStream;
    use crate::{Locator, Result};

    pub async fn connect(host: &str, port: u16) -> Result<TcpStream> {
        let conn = TcpStream::connect((host, port)).await?;
        Ok(conn)
    }

    pub async fn send_locator<W>(mut conn: W, locator: &Locator) -> Result
    where
        W: Read + Write + Unpin,
    {
        conn.write_all(locator.as_bytes()).await?;
        conn.write_all(b"\r\n").await?;
        Ok(())
    }
}

pub fn get_directory<'a>(host: &'a str, port: u16, locator: &'a Locator) -> impl Stream<Item = Result<Entry>> + 'a {
    async move {
        let mut conn = raw::connect(host, port).await?;
        raw::send_locator(&mut conn, locator).await?;

        let conn = BufReader::new(conn);
        Ok(conn.lines().map_err(<_>::from))
    }
    .try_flatten_stream()
    .take_while(|line| ready(match line {
        Ok(line) => line != ".",
        Err(_) => true,
    }))
    .and_then(|line| async move {
        let entry = line.parse::<Entry>()?;
        Ok(entry)
    })
}

pub async fn get_text_file(host: &str, port: u16, locator: &Locator) -> Result<Vec<String>> {
    let mut conn = raw::connect(host, port).await?;
    raw::send_locator(&mut conn, locator).await?;

    let conn = BufReader::new(conn);
    let lines = conn.lines()
        .take_while(|line| ready(match line {
            Ok(line) => line != ".",
            Err(_) => true,
        }))
        .map_ok(|mut line| {
            // Strip leading dot
            if line.starts_with(".") {
                line = line[1..].into();
            }

            line
        })
        .try_collect::<Vec<_>>()
        .await?;

    Ok(lines)
}

#[derive(Debug)]
pub struct Entry {
    pub kind: char,
    pub label: String,
    pub locator: Locator,
    pub host: String,
    pub port: u16,
    pub other: Vec<String>,
}

impl FromStr for Entry {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self> {
        let kind = line.chars().next().unwrap();
        let mut items = line[1..].split('\t');

        Ok(Entry {
            kind,
            label: items.next().unwrap().into(),
            locator: items.next().unwrap().parse()?,
            host: items.next().unwrap().into(),
            port: items.next().unwrap().parse()?,
            other: items.map(<_>::into).collect(),
        })
    }
}
