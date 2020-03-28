//! ITM protocol.

use crate::cli;
use anyhow::Result;
use smallvec::SmallVec;
use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io,
    io::{Read, Stdout, Write},
    ops::{Generator, GeneratorState},
    path::Path,
    pin::Pin,
    thread,
    thread::JoinHandle,
};

const PORTS_COUNT: usize = 32;

/// Run ITM parser in a child thread.
pub fn spawn(input: &Path, outputs: &[cli::MonitorOutput]) -> JoinHandle<()> {
    let input = input.to_path_buf();
    let outputs = outputs.to_vec();
    thread::spawn(move || {
        let outputs = Output::open_all(&outputs).unwrap();
        let mut parser = Parser::new(&outputs).unwrap();
        for byte in File::open(input).unwrap().bytes() {
            parser.pump(byte.unwrap()).unwrap();
        }
    })
}

struct Output<'cli> {
    ports: &'cli [u32],
    output: RefCell<Stream>,
}

enum Stream {
    Stdout(Stdout),
    File(File),
}

impl<'cli> Output<'cli> {
    fn open_all(outputs: &'cli [cli::MonitorOutput]) -> io::Result<Vec<Output<'cli>>> {
        outputs
            .iter()
            .map(|cli::MonitorOutput { ports, path }| {
                if path.is_empty() {
                    Ok(Stream::Stdout(io::stdout()))
                } else {
                    OpenOptions::new().write(true).open(path).map(Stream::File)
                }
                .map(|output| Self { ports, output: RefCell::new(output) })
            })
            .collect()
    }
}

impl Stream {
    fn write(&mut self, data: &[u8]) -> Result<()> {
        match self {
            Self::Stdout(stdout) => write_stream(stdout, data),
            Self::File(file) => write_stream(file, data),
        }
    }
}

fn write_stream<T: Write>(stream: &mut T, data: &[u8]) -> Result<()> {
    stream.write_all(data)?;
    stream.flush()?;
    Ok(())
}

struct Parser<'cli>(Pin<Box<dyn Generator<u8, Yield = (), Return = Result<!>> + 'cli>>);

enum Timestamp {
    Local { tc: u8 },
    Global1,
    Global2,
}

type Streams<'cli> = SmallVec<[&'cli RefCell<Stream>; 2]>;

impl<'cli> Parser<'cli> {
    fn new(outputs: &'cli [Output<'cli>]) -> Result<Self> {
        let gen = Box::pin(parser(outputs));
        let mut parser = Self(gen);
        parser.resume(0)?;
        Ok(parser)
    }

    fn pump(&mut self, byte: u8) -> Result<()> {
        log::debug!("BYTE 0b{0:08b} 0x{0:02X} {1:?}", byte, char::from(byte));
        self.resume(byte)
    }

    fn resume(&mut self, byte: u8) -> Result<()> {
        match self.0.as_mut().resume(byte) {
            GeneratorState::Yielded(()) => Ok(()),
            GeneratorState::Complete(Err(err)) => Err(err),
        }
    }
}

fn outputs_map<'cli>(outputs: &'cli [Output<'cli>]) -> [Streams<'cli>; PORTS_COUNT] {
    let mut map: [Streams<'_>; PORTS_COUNT] = Default::default();
    for Output { ports, output } in outputs {
        if ports.is_empty() {
            for outputs in &mut map {
                outputs.push(output);
            }
        } else {
            for port in *ports {
                map[*port as usize].push(output);
            }
        }
    }
    map
}

#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
fn parser<'cli>(
    outputs: &'cli [Output<'cli>],
) -> impl Generator<u8, Yield = (), Return = Result<!>> + 'cli {
    fn recycle<'a, T>(bytes: &'a mut SmallVec<[u8; 16]>, payload: T)
    where
        T: IntoIterator<Item = &'a u8>,
        T::IntoIter: DoubleEndedIterator,
    {
        for &byte in payload.into_iter().rev() {
            bytes.push(byte);
        }
    }
    let outputs = outputs_map(outputs);
    let mut bytes = SmallVec::<[u8; 16]>::new();
    static move |_| loop {
        bytes.push(yield);
        while let Some(byte) = bytes.pop() {
            if byte == 0 {
                let mut zeros = 8;
                let mut payload = SmallVec::<[u8; 8]>::new();
                loop {
                    let byte = yield;
                    payload.push(byte);
                    zeros += byte.trailing_zeros();
                    if byte != 0 {
                        if zeros >= 47 {
                            synchronization_packet(zeros);
                        } else {
                            log::warn!("Bad synchronization packet with {} zeros", zeros);
                            recycle(&mut bytes, &payload);
                        }
                        break;
                    }
                }
            } else if byte == 0b0111_0000 {
                log::warn!("Overflow");
            } else if byte & 0b0000_1011 == 0b0000_1000 {
                let sh = byte << 5 >> 7;
                let ex = byte << 1 >> 5;
                if byte >> 7 == 0 {
                    extension_packet(sh, ex, &[]);
                    continue;
                }
                let mut payload = SmallVec::<[u8; 4]>::with_capacity(4);
                loop {
                    let byte = yield;
                    payload.push(byte);
                    if byte >> 7 == 0 {
                        extension_packet(sh, ex, &payload);
                        break;
                    } else if payload.len() == 4 {
                        log::warn!("Bad extension packet");
                        recycle(&mut bytes, &payload);
                        break;
                    }
                }
            } else if byte & 0b0000_1011 == 0 {
                let kind = if byte & 0b1000_1111 == 0
                    && byte & 0b0111_0000 != 0b0000_0000
                    && byte & 0b0111_0000 != 0b0111_0000
                {
                    let payload = byte << 1 >> 5;
                    timestamp_packet(&Timestamp::Local { tc: 0 }, &[payload]);
                    continue;
                } else if byte & 0b1100_1111 == 0b1100_0000 {
                    let tc = byte << 2 >> 6;
                    Timestamp::Local { tc }
                } else if byte == 0b1001_0100 {
                    Timestamp::Global1
                } else if byte == 0b1011_0100 {
                    Timestamp::Global2
                } else {
                    log::warn!("Invalid header");
                    continue;
                };
                let mut payload = SmallVec::<[u8; 4]>::with_capacity(4);
                loop {
                    let byte = yield;
                    payload.push(byte);
                    if byte >> 7 == 0 {
                        timestamp_packet(&kind, &payload);
                        break;
                    } else if payload.len() == 4 {
                        log::warn!("Bad local timestamp packet");
                        recycle(&mut bytes, &payload);
                        break;
                    }
                }
            } else {
                let software = byte & 0b100 == 0;
                let address = byte >> 3;
                let size = match byte & 0b11 {
                    0b01 => 1,
                    0b10 => 2,
                    0b11 => 4,
                    _ => {
                        log::warn!("Invalid header");
                        continue;
                    }
                };
                let mut payload = SmallVec::<[u8; 4]>::with_capacity(size);
                while payload.len() < size {
                    payload.push(yield);
                }
                source_packet(software, address, &payload, &outputs)?;
            }
        }
        bytes.shrink_to_fit();
    }
}

fn synchronization_packet(zeros: u32) {
    log::debug!("Synchronized with {} zeros", zeros);
}

fn extension_packet(sh: u8, ex: u8, payload: &[u8]) {
    log::debug!("Extension packet sh={}, ex={}, payload={:?}", sh, ex, payload);
}

fn timestamp_packet(timestamp: &Timestamp, payload: &[u8]) {
    match timestamp {
        Timestamp::Local { tc } => {
            log::debug!("Local timestamp tc={}, ts={:?}", tc, payload);
        }
        Timestamp::Global1 => {
            log::debug!("Global timestamp 1 ts={:?}", payload);
        }
        Timestamp::Global2 => {
            log::debug!("Global timestamp 2 ts={:?}", payload);
        }
    }
}

fn source_packet(software: bool, port: u8, payload: &[u8], outputs: &[Streams<'_>]) -> Result<()> {
    log::debug!(
        "{} packet {:?} {:?}",
        if software { "Software" } else { "Hardware" },
        payload,
        String::from_utf8_lossy(payload)
    );
    for output in &outputs[port as usize] {
        output.borrow_mut().write(payload)?;
    }
    Ok(())
}
