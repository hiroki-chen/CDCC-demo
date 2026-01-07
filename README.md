# CDCC Demo

A confidential computing platform for secure data processing using Intel TDX and WebAssembly sandboxing.

## Overview

This project demonstrates secure computation on encrypted data within Intel TDX-protected environments. Applications are compiled to WebAssembly and executed in isolated sandboxes with policy enforcement and remote attestation.

## Key Components

- **policy-styx-server** - Main server handling requests and WASM execution
- **policy-styx-lib** - Core library (crypto, policies, TDX attestation)
- **policy-styx-web** - React-based web interface
- **sandbox-apps** - WASM applications (Polars demo, Cox regression, policy engine)
- **policy-styx-data-owner** - Data owner component with quote verification

## Quick Start

### Prerequisites

- Intel CPU with TDX support
- Rust toolchain (see `rust-toolchain.toml`)
- Node.js >= 16 (for web interface)
- TDX services (QGS, PCCS) - see [docs/host_preparation.md](docs/host_preparation.md)

### Build & Run

```bash
# Build backend
cargo build --release

# Run server
cargo run --bin policy-styx-server # note this must be done inside the TDX VM.

# Build and run web interface
cd policy-styx-web
npm install
npm run start
```

### Build WASM Apps

```bash
cd sandbox-apps/polars-demo
cargo build --target wasm32-wasip1 --release
```

## Usage Workflow

### 1. Start the Server and Web Interface

Ensure both the backend server and web interface are running:

```bash
# Terminal 1: Run server
cargo run --bin policy-styx-server

# Terminal 2: Run web UI
cd policy-styx-web
npm run start

# Terminal 3: Run the data owner (for quote verification) backend.
cargo run --bin policy-styx-data-owner
```

### 2. Access the Web Interface

Navigate to `http://localhost:3000` and create a new computation job.

### 3. Verify the Environment (Attestation)

Click "Verify Environment" to perform TDX attestation. This establishes trust in the secure environment by:
- Generating a quote from the TDX module
- Verifying the platform's integrity
- Establishing an encrypted session

### 4. Upload Files

Once attestation succeeds, upload three required files:

- **Data File** - The encrypted dataset to process (see Data Format below)
- **Program File** - The WASM binary (e.g., `polars-demo.wasm` from `sandbox-apps/polars-demo/target/wasm32-wasip1/release/`)
- **Policy Engine** - The WASM policy enforcement binary

Click "Upload Files" to securely transfer them to the server.

#### Data Format & Security Flow

The data file must be encrypted using AES-GCM encryption and packaged in the `PcdEncData` format:

1. **Raw Data**: Can be JSON, CSV, Parquet, or any binary format
   - Example: `data/ExactTravelTimeDatafromAllMatrix.json` contains healthcare data in JSON format
   
2. **Encryption Process** (Data Owner Backend):
   ```rust
   // 1. Data owner derives session key via ECDH
   // 2. Raw data is wrapped in PcdPayload
   // 3. Encrypted with AES-GCM using session key
   // 4. Packaged as PcdEncData:
   //    - protocol_version
   //    - owner_id (UUID)
   //    - encrypted_payload (ciphertext + 12-byte nonce)
   ```

3. **Security Flow** (Zero-Knowledge Server):
   ```
   Data Owner → [Encrypted Data] → Server → [Encrypted Data] → WASM Sandbox
                                              ↓
                                        [Session Key]
                                              ↓
                                        WASM Decrypts & Processes
   ```
   
   **Important**: The server NEVER decrypts the data. Only the WASM sandbox has access to both the encrypted data and the session key, ensuring the server remains zero-knowledge.

4. **Example**: See `policy-styx-lib/src/dataset.rs` for `pcd_dataset_pack_data()` function

The system supports both single datasets and multiple named datasets (HashMap format).

### 5. Execute Computation

Click "Start Secure Computation" to:
- Load the WASM application in the sandbox
- Apply policy enforcement
- Process the encrypted data
- Return the result

### Alternative: CLI Workflow

You can also use the data owner component directly:

```bash
# Run the data owner component
cargo run --bin policy-styx-data-owner

# This will handle attestation and file uploads programmatically
```

## Features

- 🔒 Hardware-based isolation with Intel TDX
- ✅ Remote attestation for integrity verification
- 🔐 End-to-end encryption (AES-GCM)
- 📋 Policy-based access control
- 🧩 WebAssembly sandboxed execution
- 🌐 Modern web interface (React 19 + TypeScript)

## Architecture

```
┌──────────────┐
│   Web UI     │ (React + TypeScript)
└──────┬───────┘
       │ HTTP
┌──────┴────────────────┐
│  Policy Styx Server   │ (Rust + Axum)
│  - TDX Attestation    │
│  - Policy Engine      │
│  - Crypto Operations  │
└──────┬────────────────┘
       │ Wasmtime
┌──────┴────────────────┐
│  WASM Applications    │
│  (Sandboxed)          │
└───────────────────────┘
```

## Documentation

- [Host Preparation](docs/host_preparation.md) - TDX setup guide
- [Sandbox Apps](sandbox-apps/README.md) - WASM app development

## License

MIT License

TODO: The uploaded data format is wrong; it should be a packet dataset consisting of tables and their parquet content.