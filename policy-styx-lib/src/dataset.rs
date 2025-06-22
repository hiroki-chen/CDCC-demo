use anyhow::{anyhow, Ok};
use uuid::Uuid;
use wasmtime::Result;

use crate::app::PcdWasmRuntime;
use crate::crypto::pcd_crypto_backend_aes_gcm_decrypt;
use crate::data::{PcdDataset, PcdEncData};
use crate::types::PcdWasmPtr;

impl PcdWasmRuntime {
    /// Add a new data to the dataset.
    ///
    /// # Note
    ///
    /// This function is meant to be called on the host side when the application is loaded and
    /// we need to add data to the dataset for further processing by the WASM module. The input
    /// data is temporarily encrypted to prevent any potential misuse. Once the policy check and
    /// other stuff is done, the data is decrypted and shared with the WASM module.
    pub fn pcd_dataset_add_data(
        &mut self,
        session_id: &Uuid,
        input_data: PcdEncData,
    ) -> Result<Uuid> {
        let (app_idx, plaintext) = self.pcd_dataset_prepare(session_id, &input_data)?;

        // Pack the plaintext into a payload.
        let payload = bincode::deserialize(&plaintext)?;
        let dataset = PcdDataset {
            dataset_policy_passed: false,
            data_count: 0,
            data_max_count: 0,
            policy_type: input_data.owner_id,
            payload_ptr: payload,
        };
        // Serialize the dataset to bytes.
        let dataset_bytes = bincode::serialize(&dataset)?;

        // Allocate the memory and write to it.
        let ptr = self.write_memory(Some(app_idx), &dataset_bytes)?;

        // add to the policy engine.
        let func = {
            let lock = self.store.data().policy_engine.clone();
            let policy_engine = lock.read().unwrap();
            let policy_engine = policy_engine
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Policy engine not found"))?;

            policy_engine
                .instance
                .get_typed_func::<PcdWasmPtr, PcdWasmPtr>(&mut self.store, "pcd_dataset_add_data")
                .map_err(|_| anyhow::anyhow!("Function 'pcd_dataset_add_data' not found"))?
        };

        let fatptr = func.call(&mut self.store, ptr)?;

        // Read the result back from the WASM module.
        let result = self.read_memory(Some(app_idx), fatptr)?;
        if result.len() != 16 {
            return Err(anyhow::anyhow!(
                "Invalid result length: expected 16, got {}",
                result.len()
            ));
        }

        // Convert the result bytes back to a UUID.
        let uuid = Uuid::from_slice(&result)
            .map_err(|_| anyhow::anyhow!("Failed to convert result bytes to UUID"))?;

        println!("[Host] Added data to dataset with UUID: {}", uuid);

        // Deallocate the memory used for the dataset.
        self.deallocate_in_app_memory(None, ptr)?;
        Ok(uuid)
    }

    fn pcd_dataset_prepare(
        &mut self,
        session_id: &Uuid,
        input_data: &PcdEncData,
    ) -> Result<(usize, Vec<u8>)> {
        let session = self
            .store
            .data()
            .sessions
            .get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        let app_idx = session.app_idx.ok_or_else(|| {
            anyhow::anyhow!("Session does not have an associated application index")
        })?;
        let key = &session.key;
        let nonce = &input_data.encrypted_payload[..input_data.encrypted_payload.len() - 16];
        let plaintext =
            pcd_crypto_backend_aes_gcm_decrypt(&input_data.encrypted_payload, key, nonce)?;
        Ok((app_idx, plaintext))
    }

    /// For complex data structures like strings, we write them into the linear memory
    /// of a specific WASM module instance.
    ///
    /// This function:
    /// 1. Calls the exported `allocate` function in the specified Wasm module.
    /// 2. Writes the string data into the memory region returned by `allocate`.
    /// 3. Returns a (pointer, length) tuple, which can be used as `Val`s for a subsequent function call.
    ///
    /// # Arguments
    /// * `app_idx` - The index of the application in the `app_registry`.
    /// * `data` - The string slice to write to the Wasm module's memory.
    ///
    /// # Returns
    /// A `Result` containing a tuple of `(u32, u32)` representing the pointer and length.
    pub fn write_memory(&mut self, app_idx: Option<usize>, data: &[u8]) -> Result<PcdWasmPtr> {
        let lock = self.store.data().policy_engine.clone();
        let policy_engine = lock.read().unwrap();

        let app = match app_idx {
            Some(app_idx) => self
                .app_registry
                .get(app_idx)
                .ok_or(anyhow!("App not found"))?,
            None => policy_engine
                .as_ref()
                .ok_or(anyhow!("Policy engine not found"))?,
        };

        let allocate_fn = app
            .instance
            .get_typed_func::<u32, u32>(&mut self.store, "allocate")
            .map_err(|_| anyhow!("Allocate function not found"))?;
        let memory = app
            .instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| anyhow!("Memory not found"))?;

        // Call the guest's allocate function to get the pointer to a suitable memory block.
        let ptr = allocate_fn.call(&mut self.store, data.len() as _)? as usize;
        memory.write(&mut self.store, ptr, data)?;

        Ok(ptr as _)
    }

    pub fn read_memory(&mut self, app_idx: Option<usize>, ptr: PcdWasmPtr) -> Result<Vec<u8>> {
        let lock = self.store.data().policy_engine.clone();
        let policy_engine = lock.read().unwrap();

        let len = ptr & 0xFFFFFFFF;
        let ptr = ptr << 32;

        let app = match app_idx {
            Some(app_idx) => self
                .app_registry
                .get(app_idx)
                .ok_or(anyhow!("App not found"))?,
            None => policy_engine
                .as_ref()
                .ok_or(anyhow!("Policy engine not found"))?,
        };

        let memory = app
            .instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| anyhow!("Memory not found"))?;

        let mut buffer = vec![];

        memory.read(&self.store, ptr as usize, &mut buffer)?;
        if buffer.len() < len as usize {
            return Err(anyhow!(
                "Read memory length {} is less than expected {}",
                buffer.len(),
                len
            ));
        }

        // Return only the requested length of data.
        Ok(buffer[..len as usize].to_vec())
    }

    /// After using the string in Wasm, the host should deallocate the memory to prevent leaks.
    ///
    /// # Arguments
    /// * `app_idx` - The index of the application where the memory was allocated.
    /// * `ptr` - The pointer to the memory region to free.
    /// * `len` - The length of the memory region.
    pub fn deallocate_in_app_memory(
        &mut self,
        app_idx: Option<usize>,
        ptr: PcdWasmPtr,
    ) -> Result<()> {
        let lock = self.store.data().policy_engine.clone();
        let policy_engine = lock.read().unwrap();

        let len = ptr & 0xFFFFFFFF;

        let app = match app_idx {
            Some(app_idx) => self
                .app_registry
                .get(app_idx)
                .ok_or(anyhow!("App not found"))?,
            None => policy_engine
                .as_ref()
                .ok_or(anyhow!("Policy engine not found"))?,
        };

        let deallocate_fn = app
            .instance
            .get_typed_func::<PcdWasmPtr, ()>(&mut self.store, "deallocate")
            .map_err(|e| anyhow!("Failed to find 'deallocate' function: {}", e))?;

        deallocate_fn.call(&mut self.store, ptr)?;

        println!(
            "[Host] Deallocated {} bytes in Wasm app #{:?} at address {}",
            len, app_idx, ptr
        );

        Ok(())
    }
}
