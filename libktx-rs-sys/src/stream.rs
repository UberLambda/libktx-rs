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

pub trait RWSeekable: Read + Write + Seek {}

impl<T: Read + Write + Seek> RWSeekable for T {}

impl<'a> Debug for dyn RWSeekable + 'a {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RWSeekable({:p})", self)
    }
}

/// A reference to a `RWSeekable`.
///
/// Pointers/references to Rust DSTs are "fat"; twice the size of a normal pointer (and possibly more in the future).  
/// As such, **transmuting C pointers to Rust pointers is not generally possible**.  
/// This struct fixes the problem by adding an extra layer of indirection:
/// C pointer -> RWSeekableRef in the heap -> T: RWSeekable in the heap
#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
struct RWSeekableRef<'a, T: RWSeekable + ?Sized + 'a> {
    ptr: *mut T,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, T: RWSeekable + ?Sized + 'a> RWSeekableRef<'a, T> {
    fn new(inner: Box<T>) -> Self {
        let ptr = Box::into_raw(inner);
        RWSeekableRef {
            ptr,
            phantom: PhantomData,
        }
    }
}

impl<'a, T: RWSeekable + ?Sized> Drop for RWSeekableRef<'a, T> {
    fn drop(&mut self) {
        // SAFETY: `self.ptr` should always have come from the `Box::into_raw()`
        //         call in `new()`, so it should always be fine to reconstruct the box here.
        let inner = unsafe { Box::from_raw(self.ptr) };
        std::mem::drop(inner)
    }
}

#[allow(unused)]
pub struct RustKtxStream<'a> {
    inner_ref: *mut RWSeekableRef<'a, dyn RWSeekable + 'a>,
    ktx_stream: Box<ktxStream>,
    ktx_phantom: PhantomData<&'a ktxStream>,
}

impl<'a> RustKtxStream<'a> {
    pub fn new(inner: Box<dyn RWSeekable + 'a>) -> Result<Self, ktx_error_code_e> {
        let boxed_inner_ref = Box::new(RWSeekableRef::new(inner));
        let inner_ref = Box::into_raw(boxed_inner_ref);

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
                    address: inner_ref as *mut c_void,
                    allocatorAddress: std::ptr::null_mut(),
                    size: 0,
                },
            },
            readpos: 0,
        });

        Ok(Self {
            inner_ref,
            ktx_stream,
            ktx_phantom: PhantomData,
        })
    }

    pub fn ktx_stream(&self) -> *mut ktxStream {
        // SAFETY: Actually safe.
        //         The C API never mutates the inner pointer, just the pointed-to struct.
        unsafe { std::mem::transmute(&*self.ktx_stream) }
    }
}

impl<'a> Drop for RustKtxStream<'a> {
    fn drop(&mut self) {
        // SAFETY: `self.inner_ref` should always have come from the `Box::into_raw()`
        //         call in `new()`, so it should always be fine to reconstruct the box here.
        let inner_ref = unsafe { Box::from_raw(self.inner_ref) };
        std::mem::drop(inner_ref)
    }
}

impl<'a> Debug for RustKtxStream<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RustKtxStream(inner={:?}, ktxStream={:p})",
            self.inner_ref, self.ktx_stream
        )
    }
}

/// Get back a reference to the [`RWSeekable`] we (indirectly, through [`RWSeekableRef`]
/// put in `ktxStream.data.custom_ptr.address`.
/// SAFETY: UB if `str` is not actually a pointer to a [`RustKtxStream`].
unsafe fn inner_rwseekable<'a>(str: *mut ktxStream) -> &'a mut dyn RWSeekable {
    let ktx_mem = (*str).data.custom_ptr.address;
    let inner_ref = std::mem::transmute::<_, *mut RWSeekableRef<dyn RWSeekable + 'a>>(ktx_mem);
    &mut *((*inner_ref).ptr)
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
