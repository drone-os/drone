//! Drone Stream definitions.

#![feature(const_maybe_uninit_zeroed)]
#![warn(missing_docs, unsafe_op_in_unsafe_fn)]
#![warn(clippy::pedantic)]
#![no_std]

use core::mem::{size_of, MaybeUninit};

/// Maximum number of streams.
pub const STREAM_COUNT: u8 = 32;

/// Length of the bootstrap sequence. See [`BOOTSTRAP_SEQUENCE`].
pub const BOOTSTRAP_SEQUENCE_LENGTH: usize = 16;

/// Sequence to bootstrap Drone Stream runtime immediately after reset.
// Generated with the following command:
// rust-script --dep rand -e 'use rand::Rng; let mut a = [0_u8; 16]; rand::thread_rng().fill(&mut a); println!("{:?}", a)'
pub const BOOTSTRAP_SEQUENCE: [u8; BOOTSTRAP_SEQUENCE_LENGTH] =
    [41, 139, 234, 244, 56, 213, 238, 162, 226, 175, 62, 199, 229, 177, 168, 74];

/// Minimal buffer size in bytes.
#[allow(clippy::cast_possible_truncation)]
pub const MIN_BUFFER_SIZE: u32 = (BOOTSTRAP_SEQUENCE_LENGTH + size_of::<Runtime>()) as _;

/// Drone Stream runtime data structure.
///
/// This structure is accessible by both the application and the debug probe.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Runtime {
    /// Streams mask. If `n`-th bit is `1`, `n`-th stream is enabled.
    pub mask: u32,
    /// TODO
    pub read_offset: u32,
    /// TODO
    pub write_offset: u32,
}

impl Runtime {
    /// Creates a new zeroed Drone Stream runtime.
    #[must_use]
    pub const fn zeroed() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}
