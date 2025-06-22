use uuid::Uuid;
#[cfg(feature = "runtime")]
use wasmtime::{Instance, Module};

pub type PcdResult<T> = anyhow::Result<T>;
pub type PcdIdentity = Uuid;
pub type PcdWasmPtr = u64; /* FAT PTR: data_ptr | data_len */

#[cfg(feature = "runtime")]
pub type PcdModule = Module;
#[cfg(feature = "runtime")]
pub type PcdInstance = Instance;
