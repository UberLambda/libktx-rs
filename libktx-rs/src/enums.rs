use crate::sys;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum StorageFormat {
    CreateNoStorage = sys::ktxTextureCreateStorageEnum_KTX_TEXTURE_CREATE_NO_STORAGE,
    CreateAllocStorage = sys::ktxTextureCreateStorageEnum_KTX_TEXTURE_CREATE_ALLOC_STORAGE,
}
