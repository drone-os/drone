use crate::cli;
use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io,
    io::{prelude::*, stdout, Stdout},
};

/// Number of streams.
pub const STREAMS_COUNT: usize = 32;

/// Opened output.
pub struct Output {
    /// Selected streams.
    src_streams: Vec<u32>,
    /// Destination stream.
    dest_stream: RefCell<DestStream>,
}

/// Destination stream.
pub enum DestStream {
    /// Standard output.
    Stdout(Stdout),
    /// File destination.
    File(File),
}

/// Output map.
pub struct OutputMap<'a>([Vec<&'a RefCell<DestStream>>; STREAMS_COUNT]);

impl Output {
    /// Opens all output streams.
    pub fn open_all(outputs: &[cli::LogOutput]) -> io::Result<Vec<Output>> {
        outputs
            .iter()
            .map(|cli::LogOutput { streams, path }| {
                if path.is_empty() {
                    Ok(DestStream::Stdout(stdout()))
                } else {
                    OpenOptions::new().write(true).open(path).map(DestStream::File)
                }
                .map(|dest_stream| Self {
                    src_streams: streams.clone(),
                    dest_stream: RefCell::new(dest_stream),
                })
            })
            .collect()
    }
}

impl<'a> From<&'a [Output]> for OutputMap<'a> {
    fn from(outputs: &'a [Output]) -> Self {
        let mut map: [Vec<&RefCell<DestStream>>; STREAMS_COUNT] = Default::default();
        for Output { src_streams, dest_stream } in outputs {
            if src_streams.is_empty() {
                for outputs in &mut map {
                    outputs.push(dest_stream);
                }
            } else {
                for src_stream in src_streams {
                    if let Some(map) = map.get_mut(*src_stream as usize) {
                        map.push(dest_stream);
                    } else {
                        log::warn!("Ignoring stream {}", src_stream);
                    }
                }
            }
        }
        OutputMap(map)
    }
}

impl OutputMap<'_> {
    /// Write `data` to all `stream` outputs.
    pub fn write(&self, stream: u8, data: &[u8]) -> io::Result<()> {
        for output in &self.0[stream as usize] {
            output.borrow_mut().write(data)?;
        }
        Ok(())
    }
}

impl DestStream {
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
