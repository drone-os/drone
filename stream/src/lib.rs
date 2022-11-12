//! Drone Stream definitions.

#![warn(missing_docs, unsafe_op_in_unsafe_fn)]
#![warn(clippy::pedantic)]
#![no_std]

use core::mem::size_of;

/// Maximum number of streams.
pub const STREAM_COUNT: u8 = 32;

/// Length of the bootstrap sequence. See [`BOOTSTRAP_SEQUENCE`].
pub const BOOTSTRAP_SEQUENCE_LENGTH: usize = 16;

/// Sequence to bootstrap Drone Stream runtime immediately after reset.
// Generated with the following command:
//
// rust-script --dep rand -e 'use rand::Rng; let mut a = [0_u8; 16]; \
// rand::thread_rng().fill(&mut a); println!("{:?}", a)'
pub const BOOTSTRAP_SEQUENCE: [u8; BOOTSTRAP_SEQUENCE_LENGTH] =
    [41, 139, 234, 244, 56, 213, 238, 162, 226, 175, 62, 199, 229, 177, 168, 74];

/// Length of one frame header.
pub const HEADER_LENGTH: u32 = 2;

/// Maximal supported length of a single transaction.
pub const MAX_TRANSACTION_LENGTH: u32 = 256;

/// Minimal buffer size in bytes.
#[allow(clippy::cast_possible_truncation)]
pub const MIN_BUFFER_SIZE: u32 = {
    let bootstrap_size =
        (BOOTSTRAP_SEQUENCE_LENGTH + size_of::<Runtime>() + size_of::<GlobalRuntime>()) as u32;
    let transaction_size = HEADER_LENGTH + MAX_TRANSACTION_LENGTH;
    let size = if bootstrap_size > transaction_size { bootstrap_size } else { transaction_size };
    (size / 4 + (size % 4 != 0) as u32) * 4
};

/// Drone Stream global runtime data structure.
///
/// This data structure risides in both the application memory and the `drone`
/// utility memory.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct GlobalRuntime {
    /// Enabled streams mask. If `n`-th bit is `1`, `n`-th stream is enabled.
    ///
    /// Writable by the probe; readable by the application.
    pub enable_mask: u32,
}

/// Drone Stream runtime data structure.
///
/// This data structure risides in both the application memory and the `drone`
/// utility memory.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Runtime {
    /// Size of the associated buffer.
    ///
    /// Read-only field.
    pub buffer_size: u32,
    /// Offset, up to which (not inclusive) the probe has read bytes.
    ///
    /// Writable by the probe; readable by the application.
    pub read_cursor: u32,
    /// Offset, up to which (not inclusive) the application has written bytes.
    ///
    /// Readable by the probe; writable by the application.
    pub write_cursor: u32,
}

impl GlobalRuntime {
    /// Creates a new zeroed Drone Stream global runtime.
    #[must_use]
    pub const fn zeroed() -> Self {
        Self { enable_mask: 0 }
    }
}

impl Runtime {
    /// Creates a new zeroed Drone Stream runtime.
    #[must_use]
    pub const fn zeroed() -> Self {
        Self { buffer_size: 0, read_cursor: 0, write_cursor: 0 }
    }
}
