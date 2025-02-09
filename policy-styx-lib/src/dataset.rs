use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasi_common::WasiCtx;
use wasmtime::Result;

use crate::app::PcdWasmRuntime;
use crate::crypto::pcd_crypto_backend_aes_gcm_decrypt;
use crate::data::{PcdEncData, PcdPayload};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

        // Who will own the secret key??
        // todo: replace them with the actual key and nonce.
        let key = b"secret_key";
        let nonce = b"nonce";
        let plaintext =
            pcd_crypto_backend_aes_gcm_decrypt(&input_data.encrypted_payload, key, nonce)?;

        // Pack the plaintext into a payload.
        let payload = bincode::deserialize(&plaintext)?;
        let dataset = PcdDataset {
            dataset_policy_passed: false,
            data_count: 0,
            data_max_count: 0,
            policy_type: input_data.owner_id,
            payload_ptr: payload,
        };

        self.data_registry.insert(uuid, dataset);
        Ok(uuid)
    }
}

// ========= WASM HOST IMPORTS ========= //

pub fn pcd_dataset_access(
    mut caller: wasmtime::Caller<'_, WasiCtx>,
    ctx: i64,
    data_uuid: u32,
    buf: u32,
    buf_len: u32,
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

    match runtime.pcd_dataset_access(&data_uuid) {
        Ok(dataset) => {
            // Copy into the memory and return the pointer.
            let data = bincode::serialize(dataset).unwrap();

            if data.len() > buf_len as usize {
                eprintln!("Buffer is too small to hold the data!");
                return -1;
            }

            memory.write(&mut runtime.store, buf as _, &data).unwrap();
            0
        },
        Err(e) => {
            eprintln!("Failed to access the dataset: {e}");
            -1
        },
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

pub fn pcd_dataset_add_data(
    mut caller: wasmtime::Caller<'_, WasiCtx>,
    ctx: i64,
    input_data: u32,
    input_data_len: u32,
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

    let input_data = match runtime.read(&memory, input_data as usize, input_data_len as usize) {
        Ok(data) => data,
        Err(_) => return -1,
    };

    let input_data = match bincode::deserialize(&input_data) {
        Ok(data) => data,
        Err(_) => return -1,
    };

    match runtime.pcd_dataset_add_data(input_data) {
        Ok(uuid) => {
            // Copy the UUID into the memory.
            let data = uuid.as_bytes();
            memory
                .write(&mut runtime.store, data_uuid as _, data)
                .unwrap();

            0
        },
        Err(_) => -1,
    }
}
