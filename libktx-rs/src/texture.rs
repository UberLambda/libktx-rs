use crate::{sys, KtxError};
use std::marker::PhantomData;

pub trait TextureSource<'a> {
    fn create_texture(self) -> Result<Texture<'a>, KtxError>;
}

pub struct Texture<'a> {
    // Not actually dead - there most likely are raw pointers referencing this!!
    #[allow(dead_code)]
    pub(crate) source: Box<dyn TextureSource<'a> + 'a>,
    pub(crate) handle: *mut sys::ktxTexture,
    pub(crate) handle_phantom: PhantomData<&'a sys::ktxTexture>,
}

impl<'a> Texture<'a> {
    pub fn new<S>(source: S) -> Result<Self, KtxError>
    where
        S: TextureSource<'a>,
    {
        source.create_texture()
    }

    /// Returns the pointer to the (C-allocated) underlying [`sys::ktxTexture`].
    ///
    /// **SAFETY**: Pointers are harmless. Dereferencing them is not!
    pub fn handle(&self) -> *mut sys::ktxTexture {
        self.handle
    }

    /// Returns the total size of image data, in bytes.
    pub fn data_size(&self) -> usize {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { sys::ktxTexture_GetDataSize(self.handle) as usize }
    }

    /// Returns a read-only view on the image data.
    pub fn data(&self) -> &[u8] {
        let data = unsafe { sys::ktxTexture_GetData(self.handle) };
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { std::slice::from_raw_parts(data, self.data_size()) }
    }

    /// Returns a read-write view on the image data.
    pub fn data_mut(&mut self) -> &mut [u8] {
        let data = unsafe { sys::ktxTexture_GetData(self.handle) };
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { std::slice::from_raw_parts_mut(data, self.data_size()) }
    }

    /// Returns the pitch (in bytes) of an image row at the specified image level.  
    /// This is rounded up to 1 if needed.
    pub fn row_pitch(&self, level: u32) -> usize {
        // SAFETY: Safe if `self.handle` is sane.
        //         `level` is not used for indexing internally; no bounds-checking required.
        unsafe { sys::ktxTexture_GetRowPitch(self.handle, level) as usize }
    }

    /// Returns the size (in bytes) of an element of the image.
    pub fn element_size(&self) -> usize {
        // SAFETY: Safe if `self.handle` is sane.
        unsafe { sys::ktxTexture_GetElementSize(self.handle) as usize }
    }
}

impl<'a> Drop for Texture<'a> {
    fn drop(&mut self) {
        unsafe {
            let vtbl = (*self.handle).vtbl;
            if let Some(destroy_fn) = (*vtbl).Destroy {
                (destroy_fn)(self.handle as *mut sys::ktxTexture);
            }
        }
    }
}
