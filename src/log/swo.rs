//! ARM® Single Wire Output protocol.

use super::{Output, OutputMap};
use anyhow::Result;
use std::{ops::Generator, pin::Pin};

enum Timestamp {
    Local { tc: u8 },
    Global1,
    Global2,
}

/// Creates a new ITM parser.
#[allow(clippy::shadow_unrelated, clippy::too_many_lines)]
pub fn parser(
    outputs: &[Output],
) -> Pin<Box<dyn Generator<u8, Yield = (), Return = Result<!>> + '_>> {
    fn recycle(bytes: &mut Vec<u8>, payload: &[u8]) {
        for &byte in payload.iter().rev() {
            bytes.push(byte);
        }
    }
    let outputs = OutputMap::from(outputs);
    let mut payload = Vec::with_capacity(8);
    Box::pin(static move |byte: u8| {
        let mut bytes = vec![byte];
        loop {
            if let Some(byte) = bytes.pop() {
                if byte == 0 {
                    let mut zeros = 8;
                    payload.clear();
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
                    payload.clear();
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
                    payload.clear();
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
                    payload.clear();
                    while payload.len() < size {
                        payload.push(yield);
                    }
                    source_packet(software, address, &payload, &outputs)?;
                }
            } else {
                bytes.push(yield);
            }
        }
    })
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

fn source_packet(software: bool, port: u8, payload: &[u8], outputs: &OutputMap<'_>) -> Result<()> {
    log::debug!(
        "Port {} {} packet {:?} {:?}",
        port,
        if software { "software" } else { "hardware" },
        payload,
        String::from_utf8_lossy(payload)
    );
    outputs.write(port, payload)?;
    Ok(())
}
