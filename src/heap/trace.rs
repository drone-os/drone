//! Heap trace file.

use std::{
    fs::File,
    io,
    io::{BufReader, Read},
    ops::{Generator, GeneratorState},
    pin::Pin,
};
use thiserror::Error;

/// The key used to shuffle packet bits.
pub const KEY: u32 = 0xC5AC_CE55;

const MAX_FRAME: usize = 8;

/// Heap trace file parser error.
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// Invalid frame.
    #[error("invalid frame")]
    InvalidFrame,
    /// Invalid frame sequence.
    #[error("invalid frame sequence")]
    InvalidSequence,
}

/// Heap trace file parser.
pub struct Parser {
    gen: Pin<Box<dyn Generator<Yield = Packet, Return = Result<(), Error>>>>,
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

#[derive(Default, Debug)]
struct Frame {
    buf: [u8; MAX_FRAME],
    head: usize,
    tail: usize,
}

impl Parser {
    /// Create a new [`Parser`] from file.
    pub fn new(trace_file: File) -> Result<Self, Error> {
        let reader = BufReader::new(trace_file);
        let gen = Box::pin(parser(reader));
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

impl Frame {
    fn push(&mut self, bytes: &[u8]) -> Result<(), Error> {
        if self.tail + bytes.len() > MAX_FRAME {
            return Err(Error::InvalidSequence);
        }
        for &byte in bytes {
            self.buf[self.tail] = byte;
            self.tail += 1;
        }
        Ok(())
    }

    fn pop_u32(&mut self) -> Result<u32, Error> {
        if self.head + 4 > self.tail {
            return Err(Error::InvalidSequence);
        }
        let mut value = 0;
        for _ in 0..4 {
            value <<= 8;
            value |= u32::from(self.buf[self.head]);
            self.head += 1;
        }
        Ok(value)
    }

    fn is_empty(&self) -> bool {
        self.head == self.tail
    }
}

#[allow(clippy::too_many_lines)]
fn parser<R: Read>(
    mut reader: BufReader<R>,
) -> impl Generator<Yield = Packet, Return = Result<(), Error>> {
    let mut frame = [0; 4];
    let mut alloc = Vec::new();
    let mut dealloc = Vec::new();
    let mut grow_in_place = Vec::new();
    let mut shrink_in_place = Vec::new();
    static move || {
        loop {
            reader.read_exact(&mut frame)?;
            frame = (u32::from_be_bytes(frame) ^ KEY).to_be_bytes();
            let header = frame[0];
            let payload = &frame[1..];
            log::trace!(
                "FRAME: (0x{:02X})0x{:02X}{:02X}{:02X}",
                header,
                payload[0],
                payload[1],
                payload[2]
            );
            match header {
                0xA1 => {
                    let mut frame = Frame::default();
                    frame.push(payload)?;
                    alloc.push(frame);
                }
                0xD1 => {
                    let mut frame = Frame::default();
                    frame.push(payload)?;
                    dealloc.push(frame);
                }
                0xB1 => {
                    let mut frame = Frame::default();
                    frame.push(payload)?;
                    grow_in_place.push(frame);
                }
                0xC1 => {
                    let mut frame = Frame::default();
                    frame.push(payload)?;
                    shrink_in_place.push(frame);
                }
                0xB2 => {
                    grow_in_place.last_mut().ok_or(Error::InvalidSequence)?.push(payload)?;
                }
                0xC2 => {
                    shrink_in_place.last_mut().ok_or(Error::InvalidSequence)?.push(payload)?;
                }
                0xA2 => {
                    let mut frame = alloc.pop().ok_or(Error::InvalidSequence)?;
                    if payload[0] != 0 || payload[1] != 0 {
                        break Err(Error::InvalidFrame);
                    }
                    frame.push(&payload[2..])?;
                    let size = frame.pop_u32()?;
                    if !frame.is_empty() {
                        break Err(Error::InvalidSequence);
                    }
                    log::debug!("Alloc: 0x{:08X}", size);
                    yield Packet::Alloc { size };
                }
                0xD2 => {
                    let mut frame = dealloc.pop().ok_or(Error::InvalidSequence)?;
                    if payload[0] != 0 || payload[1] != 0 {
                        break Err(Error::InvalidFrame);
                    }
                    frame.push(&payload[2..])?;
                    let size = frame.pop_u32()?;
                    if !frame.is_empty() {
                        break Err(Error::InvalidSequence);
                    }
                    log::debug!("Dealloc: 0x{:08X}", size);
                    yield Packet::Dealloc { size };
                }
                0xB3 => {
                    let mut frame = grow_in_place.pop().ok_or(Error::InvalidSequence)?;
                    if payload[0] != 0 {
                        break Err(Error::InvalidFrame);
                    }
                    frame.push(&payload[1..])?;
                    let old_size = frame.pop_u32()?;
                    let new_size = frame.pop_u32()?;
                    if !frame.is_empty() {
                        break Err(Error::InvalidSequence);
                    }
                    log::debug!("Grow: 0x{:08X} -> 0x{:08X}", old_size, new_size);
                    yield Packet::Grow { old_size, new_size };
                }
                0xC3 => {
                    let mut frame = shrink_in_place.pop().ok_or(Error::InvalidSequence)?;
                    if payload[0] != 0 {
                        break Err(Error::InvalidFrame);
                    }
                    frame.push(&payload[1..])?;
                    let old_size = frame.pop_u32()?;
                    let new_size = frame.pop_u32()?;
                    if !frame.is_empty() {
                        break Err(Error::InvalidSequence);
                    }
                    log::debug!("Shrink: 0x{:08X} -> 0x{:08X}", old_size, new_size);
                    yield Packet::Shrink { old_size, new_size };
                }
                _ => break Err(Error::InvalidFrame),
            }
        }
    }
}
