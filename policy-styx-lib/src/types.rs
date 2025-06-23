use uuid::Uuid;
#[cfg(feature = "runtime")]
use wasmtime::{AsContext, AsContextMut, Instance, Memory, Module};

pub type PcdResult<T> = anyhow::Result<T>;
pub type PcdIdentity = Uuid;

#[cfg(feature = "runtime")]
pub type PcdModule = Module;
#[cfg(feature = "runtime")]
pub type PcdInstance = Instance;

pub type PcdWasmRawPtr = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A pointer to a WASM memory region, used for passing data between the host and the WASM module.
pub struct PcdWasmPtr {
    pub(crate) ptr: u32,
    pub(crate) len: u32,
}

impl PcdWasmPtr {
    #[inline]
    pub fn new_ptr(ptr: u32) -> Self {
        Self { ptr, len: 0 }
    }

    #[inline]
    pub fn new(ptr: u32, len: u32) -> Self {
        Self { ptr, len }
    }

    #[inline]
    pub fn ptr(&self) -> u32 {
        self.ptr
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.ptr == 0 && self.len == 0
    }

    #[cfg(feature = "runtime")]
    pub fn read(&self, memory: &Memory, store: impl AsContext) -> PcdResult<Vec<u8>> {
        let mut buffer = vec![0; self.len as usize];
        memory.read(store, self.ptr as usize, &mut buffer)?;
        Ok(buffer)
    }

    #[cfg(feature = "runtime")]
    pub fn write(
        &self,
        memory: &mut Memory,
        buffer: &[u8],
        store: impl AsContextMut,
    ) -> PcdResult<()> {
        memory
            .write(store, self.ptr as usize, &buffer)
            .map_err(|e| e.into())
    }
}

impl From<PcdWasmPtr> for PcdWasmRawPtr {
    #[inline]
    fn from(ptr: PcdWasmPtr) -> Self {
        ((ptr.ptr as u64) << 32) | (ptr.len as u64)
    }
}

impl From<PcdWasmRawPtr> for PcdWasmPtr {
    #[inline]
    fn from(raw_ptr: PcdWasmRawPtr) -> Self {
        let ptr = (raw_ptr >> 32) as u32;
        let len = (raw_ptr & 0xFFFFFFFF) as u32;
        Self { ptr, len }
    }
}
