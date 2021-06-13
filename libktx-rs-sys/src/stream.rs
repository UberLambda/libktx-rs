// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

use crate::*;
use log;
use std::{
    ffi::c_void,
    fmt::Debug,
    io::{Read, Seek, SeekFrom, Write},
    marker::PhantomData,
};

/// Represents a Rust byte stream, i.e. something [`Read`], [`Write`] and [`Seek`].
pub trait RWSeekable: Read + Write + Seek {
    /// Upcasts self to a `RWSeekable` reference.
    ///
    /// This is required for getting a fat pointer to `self` to be stored in the
    /// C-managed [`ktxStream`].
    fn as_mut_dyn(&mut self) -> &mut dyn RWSeekable;
}

impl<T: Read + Write + Seek> RWSeekable for T {
    fn as_mut_dyn(&mut self) -> &mut dyn RWSeekable {
        self
    }
}

impl<'a> Debug for dyn RWSeekable + 'a {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RWSeekable({:p})", self)
    }
}

/// A Rust-based `ktxStream`, for reading from / writing to [`RWSeekable`]s.
#[allow(unused)]
pub struct RustKtxStream<'a, T: RWSeekable + ?Sized + 'a> {
    inner_ptr: Option<*mut T>,
    ktx_stream: Option<Box<ktxStream>>,
    ktx_phantom: PhantomData<&'a ktxStream>,
}

impl<'a, T: RWSeekable + ?Sized + 'a> RustKtxStream<'a, T> {
    /// Attempts to create a new Rust-based `ktxStream`, wrapping the given `inner` [`RWSeekable`].
    pub fn new(inner: Box<T>) -> Result<Self, ktx_error_code_e> {
        let inner_ptr = Box::into_raw(inner);
        // SAFETY: Safe, we just destructed a Box
        let inner_rwseekable_ptr = unsafe { (*inner_ptr).as_mut_dyn() } as *mut dyn RWSeekable;
        // SAFETY: Here be (rustc-version-dependent) dragons
        let (t_addr, vtable_addr): (*mut c_void, *mut c_void) =
            unsafe { std::mem::transmute(inner_rwseekable_ptr) };

        let ktx_stream = Box::new(ktxStream {
            read: Some(ktxRustStream_read),
            skip: Some(ktxRustStream_skip),
            write: Some(ktxRustStream_write),
            getpos: Some(ktxRustStream_getpos),
            setpos: Some(ktxRustStream_setpos),
            getsize: Some(ktxRustStream_getsize),
            destruct: Some(ktxRustStream_destruct),
            // Prevent the C API from messing with Rust structs
            closeOnDestruct: false,
            // SAFETY: This should be safe. The C API only sees an opaque handle at the end of the day.
            type_: streamType_eStreamTypeCustom,
            data: ktxStream__data {
                custom_ptr: ktxStream__custom_ptr {
                    address: t_addr,
                    allocatorAddress: vtable_addr,
                    size: 0,
                },
            },
            readpos: 0,
        });

        Ok(Self {
            inner_ptr: Some(inner_ptr),
            ktx_stream: Some(ktx_stream),
            ktx_phantom: PhantomData,
        })
    }

    /// Returns a handle to the underlying [`ktxStream`].
    ///
    /// ## Safety
    /// The returned handle is only for interaction with the C API.
    /// Do not modify this in any way if not absolutely necessary!
    pub fn ktx_stream(&self) -> *mut ktxStream {
        match &self.ktx_stream {
            // SAFETY - Safe. Even if C wants a mutable pointer.
            // This acts like a RefCell, where the normal interior mutability rules do not apply.
            Some(boxed) => unsafe { std::mem::transmute(boxed.as_ref()) },
            None => std::ptr::null_mut(),
        }
    }

    /// Returns a reference to the inner [`RWSeekable`].
    pub fn inner(&self) -> &T {
        // SAFETY: Safe if self has not been dropped
        unsafe { &*self.inner_ptr.expect("Self was destroyed") as &T }
    }

    /// Returns a mutable reference to the inner [`RWSeekable`].
    pub fn inner_mut(&mut self) -> &mut T {
        // SAFETY: Safe if self has not been dropped
        unsafe { &mut *self.inner_ptr.expect("Self was destroyed") as &mut T }
    }

    /// Zero out [`self.inner_ptr`], and re-box it to where it was before `new()`.
    fn rebox_inner_ptr(&mut self) -> Box<T> {
        // SAFETY: Safe-ish - a zeroed-out pointer is a null pointer in all supported platforms
        let moved_t = std::mem::replace(&mut self.inner_ptr, unsafe { std::mem::zeroed() });
        unsafe {
            // SAFETY: Safe - we're just reconstructing the box that was destructed in Self::new()
            Box::from_raw(moved_t.expect("Self was already destroyed"))
        }
    }

    /// Destroys self, giving back the boxed [`RWSeekable`] that was passed to [`Self::new`].
    pub fn into_inner(mut self) -> Box<T> {
        self.rebox_inner_ptr()
    }
}

impl<'a, T: RWSeekable + ?Sized + 'a> Drop for RustKtxStream<'a, T> {
    fn drop(&mut self) {
        // Firstly, this swaps self with a dummy
        let mut moved_self = std::mem::replace(
            self,
            RustKtxStream {
                inner_ptr: None,
                ktx_stream: None,
                ktx_phantom: PhantomData,
            },
        );

        // This is to mark the C-land `ktxStream` as invalid, and then to deallocate it
        if let Some(mut ktx_stream) = std::mem::replace(&mut moved_self.ktx_stream, None) {
            ktx_stream.data.custom_ptr = ktxStream__custom_ptr {
                address: std::ptr::null_mut(),
                allocatorAddress: std::ptr::null_mut(),
                size: 0xBADDA7A,
            };
            std::mem::drop(ktx_stream);
        }
        // The drop() of `ktx_stream` will do the rest

        // This is to destroy inner if `into_inner()` hasn't been called yet
        if let Some(_) = moved_self.inner_ptr {
            std::mem::drop(moved_self.rebox_inner_ptr())
        }

        // Finally, this prevents a drop cycle - IMPORTANT!
        // Note that we manually destroyed all fields above
        std::mem::forget(moved_self);
    }
}

fn format_option_ptr<T>(f: &mut std::fmt::Formatter<'_>, option: &Option<T>) -> std::fmt::Result {
    match option {
        Some(t) => write!(f, "{:p}", t),
        None => write!(f, "<none>"),
    }
}

impl<'a, T: RWSeekable + ?Sized + 'a> Debug for RustKtxStream<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RustKtxStream(inner=")?;
        format_option_ptr(f, &self.inner_ptr)?;
        write!(f, ", ktxStream=")?;
        format_option_ptr(f, &self.ktx_stream)?;
        write!(f, ")")
    }
}

/// Get back a reference to the [`RWSeekable`] we put in `ktxStream.data.custom_ptr`. on RustKtxStream construction.
/// SAFETY: UB if `str` is not actually a pointer to a [`RustKtxStream`].
unsafe fn inner_rwseekable<'a>(str: *mut ktxStream) -> &'a mut dyn RWSeekable {
    let t_addr = (*str).data.custom_ptr.address;
    let vtable_addr = (*str).data.custom_ptr.allocatorAddress;
    let fat_t_ptr = (t_addr, vtable_addr);
    let inner_ref: *mut dyn RWSeekable = std::mem::transmute(fat_t_ptr);
    &mut *inner_ref
}

// Since `#[feature(seek_stream_len)]` is unstable...
fn stream_len(seek: &mut dyn RWSeekable) -> std::io::Result<u64> {
    let old_pos = seek.stream_position()?;
    let size = seek.seek(SeekFrom::End(0))?;
    seek.seek(SeekFrom::Start(old_pos))?;
    Ok(size)
}

#[no_mangle]
unsafe extern "C" fn ktxRustStream_read(
    str: *mut ktxStream,
    dst: *mut c_void,
    count: ktx_size_t,
) -> ktx_error_code_e {
    let inner = inner_rwseekable(str);
    let buf = std::slice::from_raw_parts_mut(dst as *mut u8, count as usize);
    match inner.read_exact(buf) {
        Ok(_) => ktx_error_code_e_KTX_SUCCESS,
        Err(err) => {
            log::error!("ktxRustStream_read: {}", err);
            ktx_error_code_e_KTX_FILE_READ_ERROR
        }
    }
}

#[no_mangle]
unsafe extern "C" fn ktxRustStream_skip(
    str: *mut ktxStream,
    count: ktx_size_t,
) -> ktx_error_code_e {
    let inner = inner_rwseekable(str);
    match inner.seek(SeekFrom::Current(count as i64)) {
        Ok(_) => ktx_error_code_e_KTX_SUCCESS,
        Err(err) => {
            log::error!("ktxRustStream_skip: {}", err);
            ktx_error_code_e_KTX_FILE_SEEK_ERROR
        }
    }
}

#[no_mangle]
unsafe extern "C" fn ktxRustStream_write(
    str: *mut ktxStream,
    src: *const c_void,
    size: ktx_size_t,
    count: ktx_size_t,
) -> ktx_error_code_e {
    let inner = inner_rwseekable(str);
    let len = (size * count) as usize;
    let buf = std::slice::from_raw_parts(src as *const u8, len);
    match inner.write_all(buf) {
        Ok(_) => ktx_error_code_e_KTX_SUCCESS,
        Err(err) => {
            log::error!("ktxRustStream_write: {}", err);
            ktx_error_code_e_KTX_FILE_WRITE_ERROR
        }
    }
}

#[no_mangle]
unsafe extern "C" fn ktxRustStream_getpos(
    str: *mut ktxStream,
    pos: *mut ktx_off_t,
) -> ktx_error_code_e {
    let inner = inner_rwseekable(str);
    match inner.stream_position() {
        Ok(cur) => {
            *pos = cur as ktx_off_t;
            ktx_error_code_e_KTX_SUCCESS
        }
        Err(err) => {
            log::error!("ktxRustStream_getpos: {}", err);
            ktx_error_code_e_KTX_FILE_SEEK_ERROR
        }
    }
}

#[no_mangle]
unsafe extern "C" fn ktxRustStream_setpos(str: *mut ktxStream, off: ktx_off_t) -> ktx_error_code_e {
    let inner = inner_rwseekable(str);
    match inner.seek(SeekFrom::Start(off as u64)) {
        Ok(_) => ktx_error_code_e_KTX_SUCCESS,
        Err(err) => {
            log::error!("ktxRustStream_setpos: {}", err);
            ktx_error_code_e_KTX_FILE_SEEK_ERROR
        }
    }
}

#[no_mangle]
unsafe extern "C" fn ktxRustStream_getsize(
    str: *mut ktxStream,
    size: *mut ktx_size_t,
) -> ktx_error_code_e {
    let inner = inner_rwseekable(str);
    match stream_len(inner) {
        Ok(len) => {
            *size = len as ktx_size_t;
            ktx_error_code_e_KTX_SUCCESS
        }
        Err(err) => {
            log::error!("ktxRustStream_getsize: {}", err);
            ktx_error_code_e_KTX_FILE_SEEK_ERROR
        }
    }
}

#[no_mangle]
unsafe extern "C" fn ktxRustStream_destruct(_str: *mut ktxStream) {
    // No-op; `RustKtxStream::drop()` will do all the work.
}
