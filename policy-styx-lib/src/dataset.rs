use std::collections::HashMap;

use anyhow::{anyhow, Ok};
use uuid::Uuid;
use wasmtime::Result;

use crate::app::PcdWasmRuntime;
use crate::crypto::pcd_crypto_backend_aes_gcm_decrypt;
use crate::data::{PcdDataset, PcdEncData, PcdPayload};
use crate::types::{PcdWasmPtr, PcdWasmRawPtr};

impl PcdWasmRuntime {
    /// Add encrypted data to the dataset (zero-knowledge server).
    ///
    /// # Note
    ///
    /// This function receives encrypted data from the server and passes it directly
    /// to the WASM sandbox for decryption. The server never sees the plaintext data.
    pub fn pcd_dataset_add_data_encrypted(
        &mut self,
        session_id: &Uuid,
        encrypted_bytes: &[u8],
    ) -> Result<Uuid> {
        // Deserialize the encrypted data structure
        let input_data: PcdEncData = bincode::deserialize(encrypted_bytes)
            .map_err(|e| anyhow!("Failed to deserialize PcdEncData: {}", e))?;
        
        // Pass to the existing method
        self.pcd_dataset_add_data(session_id, input_data)
    }

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
        let (_, plaintext) = self.pcd_dataset_prepare(session_id, &input_data)?;

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
        let ptr = self.write_memory(None, &dataset_bytes)?;

        // add to the policy engine.
        let func = {
            let lock = self.store.data().policy_engine.clone();
            let policy_engine = lock.read().unwrap();
            let policy_engine = policy_engine
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Policy engine not found"))?;

            policy_engine
                .instance
                .get_typed_func::<PcdWasmRawPtr, PcdWasmRawPtr>(
                    &mut self.store,
                    "pcd_dataset_add_data",
                )
                .map_err(|_| anyhow::anyhow!("Function 'pcd_dataset_add_data' not found"))?
        };

        let fatptr = func.call(&mut self.store, ptr.into())?;

        // Read the result back from the WASM module.
        let result = self.read_memory(None, fatptr.into())?;
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
        let nonce = &input_data.encrypted_payload[input_data.encrypted_payload.len() - 12..];
        let encrypted_data =
            &input_data.encrypted_payload[..input_data.encrypted_payload.len() - 12];
        let plaintext = pcd_crypto_backend_aes_gcm_decrypt(encrypted_data, key, nonce)?;
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
        let ptr = allocate_fn.call(&mut self.store, data.len() as _)?;

        if ptr == 0 {
            return Err(anyhow!("Failed to allocate memory in Wasm app"));
        }
        if ptr > memory.data_size(&self.store) as u32 {
            return Err(anyhow!("Allocated pointer is out of bounds"));
        }

        memory.write(&mut self.store, ptr as _, data)?;

        log::info!(
            "[Host] Wrote {} bytes in Wasm app #{:?} at address 0x{:x}",
            data.len(),
            app_idx,
            ptr
        );

        Ok(PcdWasmPtr::new(ptr as u32, data.len() as u32))
    }

    pub fn read_memory(&mut self, app_idx: Option<usize>, ptr: PcdWasmPtr) -> Result<Vec<u8>> {
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

        let memory = app
            .instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| anyhow!("Memory not found"))?;

        ptr.read(&memory, &mut self.store)
            .map_err(|e| anyhow!("Failed to read memory: {}", e))
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
            .get_typed_func::<PcdWasmRawPtr, ()>(&mut self.store, "deallocate")
            .map_err(|e| anyhow!("Failed to find 'deallocate' function: {}", e))?;

        deallocate_fn.call(&mut self.store, ptr.into())?;

        println!(
            "[Host] Deallocated {} bytes in Wasm app #{:?} at address {}",
            ptr.len(),
            app_idx,
            ptr.ptr()
        );

        Ok(())
    }
}

///  Packs the data into a `PcdEncData` structure for further processing by the WASM module.
pub fn pcd_dataset_pack_data(data: &[u8], key: &[u8]) -> Result<PcdEncData> {
    if key.len() != 32 {
        return Err(anyhow::anyhow!("Key must be 32 bytes long"));
    }

    let pcd_payload = PcdPayload {
        payload: data.to_vec(),
        ..Default::default()
    };

    let data = bincode::serialize(&pcd_payload)
        .map_err(|e| anyhow::anyhow!("Failed to serialize PcdPayload: {}", e))?;

    // Encrypt the data using AES-GCM.
    let mut encrypted_payload = crate::crypto::pcd_crypto_backend_aes_gcm_encrypt(&data, key)?;

    println!("The nonce is : {:?}", encrypted_payload.1);

    let enc_data = PcdEncData {
        encrypted_payload: {
            encrypted_payload.0.extend_from_slice(&encrypted_payload.1); // Append nonce
            encrypted_payload.0
        },
        owner_id: Default::default(),
        protocol_version: Default::default(),
    };

    Ok(enc_data)
}

/// Serializes a collection of named data slices into a binary format.
///
/// This block takes an iterator of `(name, data)` pairs, where both `name` and `data`
/// are references, and converts them into owned `String` and `Vec<u8>` types respectively.
/// The resulting `HashMap<String, Vec<u8>>` is then serialized using `bincode`.
///
/// # Errors
///
/// Returns an error if serialization fails, wrapping the original error with additional context.
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// let data: Vec<(&str, &[u8])> = vec![("foo", &[1, 2, 3]), ("bar", &[4, 5])];
/// let packed_data = {
///     let data = data
///         .into_iter()
///         .map(|(name, data)| (name.to_string(), data.to_vec()))
///         .collect::<HashMap<_, _>>();
///     bincode::serialize(&data).unwrap()
/// };
/// ```
pub fn pcd_dataset_pack_data_multiple<I, T, S>(data: I, key: &[u8]) -> Result<PcdEncData>
where
    I: IntoIterator<Item = (S, T)>,
    T: AsRef<[u8]>,
    S: AsRef<str>,
{
    let packed_data = {
        let data = data
            .into_iter()
            .map(|(name, data)| (name.as_ref().to_string(), data.as_ref().to_vec()))
            .collect::<HashMap<_, _>>();

        bincode::serialize(&data).map_err(|e| anyhow::anyhow!("Failed to serialize data: {}", e))?
    };

    pcd_dataset_pack_data(&packed_data, key)
}
