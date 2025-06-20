use uuid::Uuid;
#[cfg(feature = "runtime")]
use wasmtime::{Instance, Module};

pub type PcdResult<T> = anyhow::Result<T>;
pub type PcdIdentity = Uuid;

#[cfg(feature = "runtime")]
pub type PcdModule = Module;
#[cfg(feature = "runtime")]
pub type PcdInstance = Instance;
