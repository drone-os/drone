//! UART logging.

use super::{Output, OutputMap};
use std::{fs::File, io::prelude::*};

/// Capture UART output.
pub fn capture(input: File, outputs: &[Output]) {
    let outputs = OutputMap::from(outputs);
    for byte in input.bytes() {
        outputs.write(0, &[byte.unwrap()]).unwrap();
    }
}
