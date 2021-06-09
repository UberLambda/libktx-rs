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

    pub fn handle(&self) -> *mut sys::ktxTexture {
        self.handle
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
