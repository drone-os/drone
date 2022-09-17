//! Drone Stream routing.

use drone_stream::STREAM_COUNT;
use eyre::{bail, Error, Result};
use std::{
    cell::RefCell,
    ffi::{OsStr, OsString},
    fs::{File, OpenOptions},
    io,
    io::{prelude::*, stdout, Stdout},
    os::unix::prelude::*,
    rc::Rc,
};
use tracing::warn;

/// Route description.
#[derive(Debug)]
pub struct RouteDesc {
    /// Output path.
    pub path: OsString,
    /// Selected streams.
    pub streams: Vec<u32>,
}

/// Stream output.
#[derive(Debug)]
pub enum Output {
    /// Standard output.
    Stdout(Stdout),
    /// File destination.
    File(File),
}

/// Routes map.
#[derive(Debug)]
pub struct Routes(RoutesArray);

type RoutesArray = [Vec<Rc<RefCell<Output>>>; STREAM_COUNT as usize];

impl Routes {
    /// Opens all outputs.
    pub fn open_all(route_descs: &[RouteDesc]) -> io::Result<Self> {
        let opened_routes = route_descs
            .iter()
            .map(|RouteDesc { path, streams }| {
                if path.is_empty() {
                    Ok(Output::Stdout(stdout()))
                } else {
                    OpenOptions::new().append(true).create(true).open(path).map(Output::File)
                }
                .map(|output| (streams, Rc::new(RefCell::new(output))))
            })
            .collect::<io::Result<Vec<_>>>()?;
        let mut routes: RoutesArray = Default::default();
        for (streams, output) in opened_routes {
            if streams.is_empty() {
                for outputs in &mut routes {
                    outputs.push(Rc::clone(&output));
                }
            } else {
                for stream in streams {
                    if let Some(map) = routes.get_mut(*stream as usize) {
                        map.push(Rc::clone(&output));
                    } else {
                        warn!("Ignoring stream {}", stream);
                    }
                }
            }
        }
        Ok(Self(routes))
    }

    /// Write `data` to all `stream` outputs.
    ///
    /// # Panics
    ///
    /// If `stream` exceeds the maximum number of stream.
    pub fn write(&self, stream: u8, data: &[u8]) -> io::Result<()> {
        for output in &self.0[stream as usize] {
            output.borrow_mut().write(data)?;
        }
        Ok(())
    }
}

impl Output {
    /// Write `data` to the output.
    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        fn write_stream<T: Write>(stream: &mut T, data: &[u8]) -> io::Result<()> {
            stream.write_all(data)?;
            stream.flush()?;
            Ok(())
        }
        match self {
            Self::Stdout(stdout) => write_stream(stdout, data),
            Self::File(file) => write_stream(file, data),
        }
    }
}

impl TryFrom<&[u8]> for RouteDesc {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let mut chunks = value.split(|&b| b == b':');
        let path = OsStr::from_bytes(chunks.next().unwrap()).into();
        let streams = chunks
            .map(|stream| {
                let number = String::from_utf8(stream.to_vec())?.parse()?;
                if number >= STREAM_COUNT.into() {
                    bail!(
                        "Stream number {number} exceeds the maximum number of streams \
                         {STREAM_COUNT}"
                    );
                }
                Ok(number)
            })
            .collect::<Result<_>>()?;
        Ok(Self { path, streams })
    }
}
