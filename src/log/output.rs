use crate::cli;
use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io,
    io::{prelude::*, Stdout},
};

/// Maximum ports count.
pub const PORTS_COUNT: usize = 32;

/// Opened output.
pub struct Output {
    /// Selected ports.
    ports: Vec<u32>,
    /// Output stream.
    stream: RefCell<OutputStream>,
}

/// Output stream.
pub enum OutputStream {
    /// Standard output.
    Stdout(Stdout),
    /// File output.
    File(File),
}

/// Output map.
pub struct OutputMap<'a>([Vec<&'a RefCell<OutputStream>>; PORTS_COUNT]);

impl Output {
    /// Opens all output streams.
    pub fn open_all(outputs: &[cli::LogOutput]) -> io::Result<Vec<Output>> {
        outputs
            .iter()
            .map(|cli::LogOutput { ports, path }| {
                if path.is_empty() {
                    Ok(OutputStream::Stdout(io::stdout()))
                } else {
                    OpenOptions::new().write(true).open(path).map(OutputStream::File)
                }
                .map(|stream| Self { ports: ports.clone(), stream: RefCell::new(stream) })
            })
            .collect()
    }
}

impl<'a> From<&'a [Output]> for OutputMap<'a> {
    fn from(outputs: &'a [Output]) -> Self {
        let mut map: [Vec<&RefCell<OutputStream>>; PORTS_COUNT] = Default::default();
        for Output { ports, stream } in outputs {
            if ports.is_empty() {
                for outputs in &mut map {
                    outputs.push(stream);
                }
            } else {
                for port in ports {
                    map[*port as usize].push(stream);
                }
            }
        }
        OutputMap(map)
    }
}

impl OutputMap<'_> {
    /// Write `data` to all `port` outputs.
    pub fn write(&self, port: u32, data: &[u8]) -> io::Result<()> {
        for output in &self.0[port as usize] {
            output.borrow_mut().write(data)?;
        }
        Ok(())
    }
}

impl OutputStream {
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
