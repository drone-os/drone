//! Heap trace file.

use std::fs::File;
use std::io;
use std::io::{BufReader, Read};

use thiserror::Error;
use tracing::debug;

/// Heap trace file parser error.
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// Invalid frame sequence.
    #[error("invalid frame sequence")]
    InvalidSequence,
}

/// Heap trace file parser.
pub struct Parser {
    reader: BufReader<File>,
}

/// Heap trace file packet.
pub enum Packet {
    /// Allocate a block of memory.
    Alloc {
        /// Block size.
        size: u32,
    },
    /// Deallocate a block of memory.
    Dealloc {
        /// Block size.
        size: u32,
    },
    /// Extend a memory block.
    Grow {
        /// Old block size.
        old_size: u32,
        /// New block size.
        new_size: u32,
    },
    /// Shrink a memory block.
    Shrink {
        /// Old block size.
        old_size: u32,
        /// New block size.
        new_size: u32,
    },
}

impl Parser {
    /// Create a new [`Parser`] from a file.
    pub fn new(trace_file: File) -> Result<Self, Error> {
        let reader = BufReader::new(trace_file);
        Ok(Self { reader })
    }
}

impl Iterator for Parser {
    type Item = Result<Packet, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match parse(&mut self.reader) {
            Err(Error::Io(ref err)) if err.kind() == io::ErrorKind::UnexpectedEof => None,
            packet @ Ok(_) => Some(packet),
            err @ Err(_) => Some(err),
        }
    }
}

fn parse<R: Read>(reader: &mut R) -> Result<Packet, Error> {
    let mut header = [0; 1];
    reader.read_exact(&mut header)?;
    match header[0] {
        0 => {
            let mut payload = [0; 4];
            reader.read_exact(&mut payload)?;
            let size = u32::from_le_bytes(payload);
            debug!("Alloc: 0x{:08X}", size);
            Ok(Packet::Alloc { size })
        }
        1 => {
            let mut payload = [0; 4];
            reader.read_exact(&mut payload)?;
            let size = u32::from_le_bytes(payload);
            debug!("Dealloc: 0x{:08X}", size);
            Ok(Packet::Dealloc { size })
        }
        2 => {
            let mut payload = [0; 4];
            reader.read_exact(&mut payload)?;
            let old_size = u32::from_le_bytes(payload);
            reader.read_exact(&mut payload)?;
            let new_size = u32::from_le_bytes(payload);
            debug!("Grow: 0x{:08X} -> 0x{:08X}", old_size, new_size);
            Ok(Packet::Grow { old_size, new_size })
        }
        3 => {
            let mut payload = [0; 4];
            reader.read_exact(&mut payload)?;
            let old_size = u32::from_le_bytes(payload);
            reader.read_exact(&mut payload)?;
            let new_size = u32::from_le_bytes(payload);
            debug!("Shrink: 0x{:08X} -> 0x{:08X}", old_size, new_size);
            Ok(Packet::Shrink { old_size, new_size })
        }
        _ => Err(Error::InvalidSequence),
    }
}
