//! Drone Serial Output protocol.

use super::{Output, OutputMap};
use anyhow::Result;
use std::{ops::Generator, pin::Pin};

const KEY: u8 = 0b100_1011;

/// Creates a new DSO parser.
pub fn parser(
    outputs: &[Output],
) -> Pin<Box<dyn Generator<u8, Yield = (), Return = Result<!>> + '_>> {
    let outputs = OutputMap::from(outputs);
    let mut payload = Vec::with_capacity(16);
    Box::pin(static move |mut byte| {
        loop {
            if byte >> 1 == KEY {
                let mut port = (byte & 1) << 4;
                byte = yield;
                port |= byte >> 4;
                let length = byte & 0xF;
                for _ in 0..=length {
                    payload.push(yield);
                }
                log::debug!(
                    "Port {} packet {:?} {:?}",
                    port,
                    payload,
                    String::from_utf8_lossy(&payload)
                );
                outputs.write(port, &payload)?;
                payload.clear();
            }
            byte = yield;
        }
    })
}
