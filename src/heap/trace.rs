//! Heap trace file.

use log::debug;
use std::{
    fs::File,
    io::{self, BufReader, Read},
    ops::{Generator, GeneratorState},
    pin::Pin,
};

/// The key used to shuffle packet bits.
pub const KEY: u32 = 0xC5AC_CE55;

/// Heap trace file parser error.
#[derive(Debug)]
pub enum Error {
    /// I/O error.
    Io(io::Error),
    /// Invalid packet header.
    InvalidHeader,
    /// Invalid packet value.
    InvalidValue,
}

/// Heap trace file parser.
pub struct Parser {
    gen: Pin<Box<dyn Generator<Yield = Packet, Return = Result<(), Error>>>>,
}

/// Heap trace file packet.
pub enum Packet {
    /// Allocate.
    Alloc {
        /// Allocation size.
        size: u32,
    },
    /// Deallocate.
    Dealloc {
        /// Allocation size.
        size: u32,
    },
    /// Grow in place.
    GrowInPlace {
        /// Allocation size.
        size: u32,
        /// New allocation size.
        new_size: u32,
    },
    /// Shrink in place.
    ShrinkInPlace {
        /// Allocation size.
        size: u32,
        /// New allocation size.
        new_size: u32,
    },
}

impl Parser {
    /// Create a new [`Parser`] from file.
    pub fn new(trace_file: File, big_endian: bool) -> Result<Self, Error> {
        let reader = BufReader::new(trace_file);
        let gen = Box::pin(parser(reader, big_endian));
        Ok(Self { gen })
    }
}

impl Iterator for Parser {
    type Item = Result<Packet, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.gen.as_mut().resume(()) {
            GeneratorState::Yielded(packet) => Some(Ok(packet)),
            GeneratorState::Complete(Ok(())) => None,
            GeneratorState::Complete(Err(Error::Io(ref err)))
                if err.kind() == io::ErrorKind::UnexpectedEof =>
            {
                None
            }
            GeneratorState::Complete(Err(err)) => Some(Err(err)),
        }
    }
}

fn parser<R: Read>(
    mut reader: BufReader<R>,
    big_endian: bool,
) -> impl Generator<Yield = Packet, Return = Result<(), Error>> {
    let mut header = [0; 2];
    let mut value = [0; 4];
    let parse_u32 = move |bytes| {
        let value = if big_endian { u32::from_be_bytes(bytes) } else { u32::from_le_bytes(bytes) };
        if value == 0 { Err(Error::InvalidValue) } else { Ok(value ^ KEY) }
    };
    static move || {
        loop {
            reader.read_exact(&mut header)?;
            debug!("HEADER: 0x{:02X}{:02X}", header[0], header[1]);
            match header {
                [0xAB, 0xCD] => {
                    reader.read_exact(&mut value)?;
                    let size = parse_u32(value)?;
                    yield Packet::Alloc { size };
                }
                [0xDC, 0xBA] => {
                    reader.read_exact(&mut value)?;
                    let size = parse_u32(value)?;
                    yield Packet::Dealloc { size };
                }
                [0xBC, 0xDE] => {
                    reader.read_exact(&mut value)?;
                    let size = parse_u32(value)?;
                    reader.read_exact(&mut value)?;
                    let new_size = parse_u32(value)?;
                    yield Packet::GrowInPlace { size, new_size };
                }
                [0xED, 0xCB] => {
                    reader.read_exact(&mut value)?;
                    let size = parse_u32(value)?;
                    reader.read_exact(&mut value)?;
                    let new_size = parse_u32(value)?;
                    yield Packet::ShrinkInPlace { size, new_size };
                }
                _ => break Err(Error::InvalidHeader),
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{}", err),
            Self::InvalidHeader => write!(f, "Invalid packet header"),
            Self::InvalidValue => write!(f, "Invalid packet value"),
        }
    }
}
