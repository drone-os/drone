//! Drone Stream runtime.
//!
//! This module provides access to the special area in the application memory
//! for storing the runtime state of Drone Stream.

use drone_openocd::{
    target, target_read_buffer, target_read_u32, target_write_buffer, target_write_u32, ERROR_FAIL,
    ERROR_OK,
};
use drone_stream::{GlobalRuntime, Runtime, BOOTSTRAP_SEQUENCE, BOOTSTRAP_SEQUENCE_LENGTH};
use std::cmp::Ordering;
use std::mem::{size_of, transmute, MaybeUninit};
use std::os::raw::c_int;
use std::ptr;

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

/// Methods for working with the global runtime instance that resides in the
/// application memory.
pub trait RemoteGlobalRuntime {
    /// Creates a new `GlobalRuntime` value with the given `enable_mask` field,
    /// and all other fields zeroed.
    fn from_enable_mask(enable_mask: u32) -> Self;

    /// Writes the `enable_mask` field to the target.
    ///
    /// # Safety
    ///
    /// `target` must be a valid pointer to the OpencOCD target.
    unsafe fn target_write_enable_mask(
        &self,
        target: *mut target,
        global_address: u32,
    ) -> Result<()>;
}

/// Methods for working with the runtime instance that resides in the
/// application memory.
pub trait RemoteRuntime {
    /// Creates a new `Runtime` value with the given `buffer_size` field, and
    /// all other fields zeroed.
    fn from_buffer_size(buffer_size: u32) -> Self;

    /// Writes the runtime to the target as a bootstrap sequence.
    ///
    /// # Safety
    ///
    /// `target` must be a valid pointer to the OpencOCD target.
    unsafe fn target_write_bootstrap(
        &self,
        target: *mut target,
        address: u32,
        global_runtime: Option<&GlobalRuntime>,
    ) -> Result<()>;

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
    ) -> Result<(&'b mut [u8], Option<usize>)>;
}

macro_rules! offset_of {
    ($field:ident) => {{
        let uninit = MaybeUninit::<Self>::uninit();
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
                ($address - size_of::<Self>() as u32 + offset_of!($field) as u32).into(),
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
                ($address - size_of::<Self>() as u32 + offset_of!($field) as u32).into(),
                $self.$field,
            )
        })
    }};
}

macro_rules! write_global_field {
    ($self:ident, $target:expr, $global_address:expr, $field:ident) => {{
        result_from(unsafe {
            target_write_u32(
                $target,
                ($global_address + offset_of!($field) as u32).into(),
                $self.$field,
            )
        })
    }};
}

impl RemoteGlobalRuntime for GlobalRuntime {
    fn from_enable_mask(enable_mask: u32) -> Self {
        let mut runtime = Self::zeroed();
        runtime.enable_mask = enable_mask;
        runtime
    }

    unsafe fn target_write_enable_mask(
        &self,
        target: *mut target,
        global_address: u32,
    ) -> Result<()> {
        write_global_field!(self, target, global_address, enable_mask)
    }
}

impl RemoteRuntime for Runtime {
    fn from_buffer_size(buffer_size: u32) -> Self {
        let mut runtime = Self::zeroed();
        runtime.buffer_size = buffer_size;
        runtime
    }

    unsafe fn target_write_bootstrap(
        &self,
        target: *mut target,
        address: u32,
        global_runtime: Option<&GlobalRuntime>,
    ) -> Result<()> {
        unsafe {
            let mut bootstrap_address = address.into();
            result_from(target_write_buffer(
                target,
                bootstrap_address,
                BOOTSTRAP_SEQUENCE_LENGTH as u32,
                BOOTSTRAP_SEQUENCE.as_ptr(),
            ))?;
            bootstrap_address += BOOTSTRAP_SEQUENCE_LENGTH as u64;
            let runtime: [u8; size_of::<Runtime>()] = transmute(self.clone());
            result_from(target_write_buffer(
                target,
                bootstrap_address,
                size_of::<Runtime>() as u32,
                runtime.as_ptr(),
            ))?;
            bootstrap_address += size_of::<Runtime>() as u64;
            if let Some(global_runtime) = global_runtime {
                let global_runtime: [u8; size_of::<GlobalRuntime>()] =
                    transmute(global_runtime.clone());
                result_from(target_write_buffer(
                    target,
                    bootstrap_address,
                    size_of::<GlobalRuntime>() as u32,
                    global_runtime.as_ptr(),
                ))?;
            }
            self.target_write_read_cursor(target, address)?;
            self.target_write_write_cursor(target, address)?;
        }
        Ok(())
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
    ) -> Result<(&'b mut [u8], Option<usize>)> {
        let mut count;
        let mut wrap_point = None;
        unsafe { self.target_read_write_cursor(target, address)? };
        match self.write_cursor.cmp(&self.read_cursor) {
            Ordering::Equal => return Ok((&mut buffer[0..0], wrap_point)),
            Ordering::Greater => {
                count = self.write_cursor - self.read_cursor;
                assert!(count as usize <= buffer.len());
                unsafe {
                    result_from(target_read_buffer(
                        target,
                        (address + self.read_cursor).into(),
                        count,
                        buffer.as_mut_ptr(),
                    ))?;
                }
            }
            Ordering::Less => {
                count = buffer.len() as u32 - self.read_cursor;
                assert!(count as usize <= buffer.len());
                unsafe {
                    result_from(target_read_buffer(
                        target,
                        (address + self.read_cursor).into(),
                        count,
                        buffer.as_mut_ptr(),
                    ))?;
                }
                wrap_point = Some(count as usize);
                if self.write_cursor > 0 {
                    let ptr = unsafe { buffer.as_mut_ptr().add(count as usize) };
                    count += self.write_cursor;
                    assert!(count as usize <= buffer.len());
                    unsafe {
                        result_from(target_read_buffer(
                            target,
                            address.into(),
                            self.write_cursor,
                            ptr,
                        ))?;
                    }
                }
            }
        }
        self.read_cursor = self.write_cursor;
        unsafe { self.target_write_read_cursor(target, address)? };
        Ok((&mut buffer[0..count as usize], wrap_point))
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
