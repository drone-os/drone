//! Drone Stream runtime.
//!
//! This module provides access to the special area in the application memory
//! for storing the runtime state of Drone Stream.

use std::mem::{size_of, transmute, MaybeUninit};
use std::os::raw::c_int;
use std::ptr;

use drone_openocd::{
    target, target_read_buffer, target_read_u32, target_write_buffer, target_write_u32, ERROR_FAIL,
    ERROR_OK,
};
use drone_stream::{Runtime, BOOTSTRAP_SEQUENCE, BOOTSTRAP_SEQUENCE_LENGTH, HEADER_LENGTH};

/// OpenOCD API error.
#[derive(Debug)]
pub enum Error {
    /// See "Error:" log entry for meaningful message to the user. The caller
    /// should make no assumptions about what went wrong and try to handle
    /// the problem.
    Fail,
    /// Other error.
    Other(c_int),
}

/// OpenOCD API result.
pub type Result<T> = std::result::Result<T, Error>;

/// Methods for working with the runtime instance that resides in the
/// application memory.
pub trait RemoteRuntime {
    /// Creates a new `Runtime` value with the given `enable_mask` field, and
    /// all other fields zeroed.
    fn from_enable_mask(enable_mask: u32) -> Self;

    /// Writes the runtime to the target as a bootstrap sequence.
    ///
    /// # Safety
    ///
    /// `target` must be a valid pointer to the OpencOCD target.
    unsafe fn target_write_bootstrap(&self, target: *mut target, address: u32) -> Result<()>;

    /// Writes the `enable_mask` field to the target.
    ///
    /// # Safety
    ///
    /// `target` must be a valid pointer to the OpencOCD target.
    unsafe fn target_write_enable_mask(&self, target: *mut target, address: u32) -> Result<()>;

    /// Writes the `read_cursor` field to the target.
    ///
    /// # Safety
    ///
    /// `target` must be a valid pointer to the OpencOCD target.
    unsafe fn target_write_read_cursor(&self, target: *mut target, address: u32) -> Result<()>;

    /// Writes the `write_cursor` field to the target.
    ///
    /// # Safety
    ///
    /// `target` must be a valid pointer to the OpencOCD target.
    unsafe fn target_write_write_cursor(&self, target: *mut target, address: u32) -> Result<()>;

    /// Reads the `write_cursor` field from the target.
    ///
    /// # Safety
    ///
    /// `target` must be a valid pointer to the OpencOCD target.
    unsafe fn target_read_write_cursor(&mut self, target: *mut target, address: u32) -> Result<()>;

    /// Consumes pending data available on the target.
    ///
    /// # Safety
    ///
    /// `target` must be a valid pointer to the OpencOCD target.
    unsafe fn target_consume_buffer<'r, 'b>(
        &'r mut self,
        target: *mut target,
        address: u32,
        buffer: &'b mut [u8],
    ) -> Result<&'b mut [u8]>;
}

macro_rules! offset_of {
    ($field:ident) => {{
        let uninit = MaybeUninit::<Runtime>::uninit();
        let base_ptr = uninit.as_ptr();
        let field_ptr = ptr::addr_of!((*base_ptr).$field);
        (field_ptr.cast::<u8>()).offset_from(base_ptr.cast())
    }};
}

macro_rules! read_field {
    ($self:ident, $target:expr, $address:expr, $field:ident) => {{
        result_from(unsafe {
            target_read_u32(
                $target,
                ($address - size_of::<Runtime>() as u32 + offset_of!($field) as u32).into(),
                &mut $self.$field,
            )
        })
    }};
}

macro_rules! write_field {
    ($self:ident, $target:expr, $address:expr, $field:ident) => {{
        result_from(unsafe {
            target_write_u32(
                $target,
                ($address - size_of::<Runtime>() as u32 + offset_of!($field) as u32).into(),
                $self.$field,
            )
        })
    }};
}

impl RemoteRuntime for Runtime {
    fn from_enable_mask(enable_mask: u32) -> Self {
        let mut runtime = Runtime::zeroed();
        runtime.enable_mask = enable_mask;
        runtime
    }

    unsafe fn target_write_bootstrap(&self, target: *mut target, address: u32) -> Result<()> {
        unsafe {
            result_from(target_write_buffer(
                target,
                address.into(),
                BOOTSTRAP_SEQUENCE_LENGTH as u32,
                BOOTSTRAP_SEQUENCE.as_ptr(),
            ))?;
            let runtime: [u8; size_of::<Runtime>()] = transmute(self.clone());
            result_from(target_write_buffer(
                target,
                (address + BOOTSTRAP_SEQUENCE_LENGTH as u32).into(),
                size_of::<Runtime>() as u32,
                runtime.as_ptr(),
            ))?;
            self.target_write_read_cursor(target, address)?;
            self.target_write_write_cursor(target, address)?;
        }
        Ok(())
    }

    unsafe fn target_write_enable_mask(&self, target: *mut target, address: u32) -> Result<()> {
        write_field!(self, target, address, enable_mask)
    }

    unsafe fn target_write_read_cursor(&self, target: *mut target, address: u32) -> Result<()> {
        write_field!(self, target, address, read_cursor)
    }

    unsafe fn target_write_write_cursor(&self, target: *mut target, address: u32) -> Result<()> {
        write_field!(self, target, address, write_cursor)
    }

    unsafe fn target_read_write_cursor(&mut self, target: *mut target, address: u32) -> Result<()> {
        read_field!(self, target, address, write_cursor)
    }

    unsafe fn target_consume_buffer<'r, 'b>(
        &'r mut self,
        target: *mut target,
        address: u32,
        buffer: &'b mut [u8],
    ) -> Result<&'b mut [u8]> {
        let mut count;
        unsafe { self.target_read_write_cursor(target, address)? };
        let start = (address + self.read_cursor).into();
        if self.write_cursor >= self.read_cursor {
            count = self.write_cursor - self.read_cursor;
            unsafe {
                assert!(count as usize <= buffer.len());
                result_from(target_read_buffer(target, start, count, buffer.as_mut_ptr()))?;
            }
        } else {
            count = buffer.len() as u32 - self.read_cursor;
            unsafe {
                if count > HEADER_LENGTH {
                    assert!(count as usize <= buffer.len());
                    result_from(target_read_buffer(target, start, count, buffer.as_mut_ptr()))?;
                } else {
                    count = 0;
                }
                let ptr = buffer.as_mut_ptr().add(count as usize);
                count += self.write_cursor;
                assert!(count as usize <= buffer.len());
                result_from(target_read_buffer(target, address.into(), self.write_cursor, ptr))?;
            }
        }
        self.read_cursor = self.write_cursor;
        unsafe { self.target_write_read_cursor(target, address)? };
        Ok(&mut buffer[0..count as usize])
    }
}

/// Converts OpenOCD error code into `Result`.
pub fn result_from(code: c_int) -> Result<()> {
    #[allow(clippy::cast_possible_wrap)]
    const ERROR_OK_: c_int = ERROR_OK as _;
    match code {
        ERROR_OK_ => Ok(()),
        ERROR_FAIL => Err(Error::Fail),
        err => Err(Error::Other(err)),
    }
}

/// Converts `Result` into OpenOCD error code.
#[allow(clippy::cast_possible_wrap)]
pub fn result_into(result: Result<()>) -> c_int {
    match result {
        #[allow(clippy::cast_possible_wrap)]
        Ok(()) => ERROR_OK as _,
        Err(Error::Fail) => ERROR_FAIL as _,
        Err(Error::Other(err)) => err,
    }
}
