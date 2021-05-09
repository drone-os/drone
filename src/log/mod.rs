//! Debug log interface.

pub mod swo;

mod output;

pub use self::output::{Output, OutputMap, OutputStream};

use anyhow::Result;
use std::{
    fs::File,
    io::prelude::*,
    ops::{Generator, GeneratorState},
    path::PathBuf,
    pin::Pin,
    thread,
};

type ParserFn = fn(&[Output]) -> Pin<Box<dyn Generator<u8, Yield = (), Return = Result<!>> + '_>>;

/// Runs log capture thread.
pub fn capture(input: PathBuf, outputs: Vec<Output>, parser: ParserFn) {
    thread::spawn(move || {
        (|| -> Result<()> {
            let input = File::open(input)?;
            let mut parser = Box::pin(parser(&outputs));
            for byte in input.bytes() {
                let byte = byte?;
                log::debug!("BYTE 0b{0:08b} 0x{0:02X} {1:?}", byte, char::from(byte));
                match parser.as_mut().resume(byte) {
                    GeneratorState::Yielded(()) => (),
                    GeneratorState::Complete(Err(err)) => panic!("log parser failure: {}", err),
                }
            }
            Ok(())
        })()
        .expect("log capture thread failed");
    });
}
