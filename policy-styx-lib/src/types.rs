use uuid::Uuid;
use wasmtime::{Instance, Module};

pub type PcdResult<T> = anyhow::Result<T>;
pub type PcdIdentity = Uuid;
pub type PcdModule = Module;
pub type PcdInstance = Instance;
/// The offset to the shared runtime memory with the host.
pub type PcdRuntimeOffset = isize;
