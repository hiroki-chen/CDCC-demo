use uuid::Uuid;
use wasi_common::WasiCtx;
use wasmtime::Result;

use crate::app::PcdWasmRuntime;
use crate::data::{PcdEncData, PcdPayload};

#[derive(Debug, Clone)]
pub struct PcdDataset {
    pub dataset_policy_passed: bool,
    pub data_count: u32,
    pub data_max_count: u32,
    pub policy_type: Uuid,
    pub payload_ptr: PcdPayload,
}

impl<T> PcdWasmRuntime<T> {
    #[inline]
    pub fn pcd_dataset_access(&self, uuid: &Uuid) -> Result<&PcdDataset> {
        self.data_registry
            .get(uuid)
            .ok_or_else(|| anyhow::anyhow!("dataset not found for {uuid}"))
    }

    #[inline]
    pub fn pcd_dataset_release(&mut self, uuid: &Uuid) -> Result<()> {
        self.data_registry.remove(uuid);
        Ok(())
    }

    /// Add a new data to the dataset.
    /// 
    /// # Note
    /// 
    /// This function is meant to be called on the host side when the application is loaded and
    /// we need to add data to the dataset for further processing by the WASM module. The input
    /// data is temporarily encrypted to prevent any potential misuse. Once the policy check and
    /// other stuff is done, the data is decrypted and shared with the WASM module.
    pub fn pcd_dataset_add_data(&mut self, input_data: PcdEncData) -> Result<Uuid> {
        let uuid = Uuid::new_v4();

        todo!("We have to deal with encryption here.");

        Ok(uuid)
    }
}

pub fn pcd_dataset_access(
    mut caller: wasmtime::Caller<'_, WasiCtx>,
    ctx: i64,
    data_uuid: u32,
) -> i32 {
    let runtime = unsafe { &*(ctx as *const PcdWasmRuntime<WasiCtx>) };
    // let app = match runtime.pcd_app_get_app(idx as usize) {
    //     Some(app) => app,
    //     None => return -1,
    // };
    let memory = match caller.get_export("memory") {
        Some(export) => match export.into_memory() {
            Some(memory) => memory,
            None => return -1,
        },
        None => return -1,
    };

    let data_uuid = match runtime.read(&memory, data_uuid as usize, 16) {
        Ok(data) => Uuid::from_slice(&data).unwrap(),
        Err(_) => return -1,
    };

    match runtime.pcd_dataset_access(&data_uuid) {
        Ok(dataset) => {
            // copy into the memory and returns the

            todo!()
        },
        Err(_) => -1,
    }
}

pub fn pcd_dataset_release(
    mut caller: wasmtime::Caller<'_, WasiCtx>,
    ctx: i64,
    data_uuid: u32,
) -> i32 {
    let runtime = unsafe { &mut *(ctx as *mut PcdWasmRuntime<WasiCtx>) };

    let memory = match caller.get_export("memory") {
        Some(export) => match export.into_memory() {
            Some(memory) => memory,
            None => return -1,
        },
        None => return -1,
    };

    let data_uuid = match runtime.read(&memory, data_uuid as usize, 16) {
        Ok(data) => Uuid::from_slice(&data).unwrap(),
        Err(_) => return -1,
    };

    match runtime.pcd_dataset_release(&data_uuid) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}
