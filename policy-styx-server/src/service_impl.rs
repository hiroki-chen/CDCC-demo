use std::collections::HashMap;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response, Result};
use axum::routing::post;
use axum::{Json, Router};
use p256::ecdh::EphemeralSecret;
use p256::elliptic_curve::rand_core::OsRng;
use p256::PublicKey;
use policy_styx_lib::app::{PcdWasmRuntime, PcdWasmRuntimeBuilder, Session};
#[cfg(all(not(feature = "mock"), feature = "platform-tdx"))]
use policy_styx_lib::attestation;
use policy_styx_lib::proxy;
use policy_styx_lib::types::PcdWasmRawPtr;
use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;
use tokio::sync::Mutex;
use tower_http::cors::{self, AllowOrigin, CorsLayer};
use uuid::Uuid;
use wasi_common::sync::WasiCtxBuilder;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PolicyStyxAttestationRequest {
    // base64.
    #[serde_as(as = "Base64")]
    gx: Vec<u8>, // The client's public key.
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PolicyStyxAttestationResponse {
    #[serde_as(as = "Base64")]
    gy: Vec<u8>, // The server's public key.
    #[serde_as(as = "Base64")]
    quote: Vec<u8>, // The attestation report.
    quote_type: u32,  // The type of the quote.
    session_id: Uuid, // The session ID.
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PolicyStyxUploadRequest {
    session_id: Uuid, // The session ID for the upload.
    #[serde_as(as = "Base64")]
    data: Vec<u8>, // The data to be uploaded.
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PolicyStyxPrepareRequest {
    session_id: Uuid, // The session ID for the computation.
    data_file: String,
    program_file: String,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PolicyStyxComputeRequest {
    session_id: Uuid, // The session ID for the computation.
    entry: String,
    args: HashMap<String, Vec<u8>>, // Arguments for the computation.
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PolicyStyxUploadResponse {}

impl IntoResponse for PolicyStyxAttestationResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl IntoResponse for PolicyStyxUploadResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

pub struct ServerState {
    rt: PcdWasmRuntime,
}

impl Deref for ServerState {
    type Target = HashMap<Uuid, Session>;

    fn deref(&self) -> &Self::Target {
        &self.rt.store.data().sessions
    }
}

impl DerefMut for ServerState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rt.store.data_mut().sessions
    }
}

impl ServerState {
    pub fn new() -> Result<Self> {
        let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();

        let mut rt = PcdWasmRuntimeBuilder::new()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .with_host_function(
                "pcd_dataset_data_access",
                proxy::pcd_dataset_data_access_host,
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .build(wasi_ctx);

        rt.load_policy_engine("./data/policy_engine.wasm")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(ServerState { rt })
    }
}

type Sessions = Arc<Mutex<ServerState>>;

async fn policy_styx_prepare_computation(
    State(sessions): State<Sessions>,
    Json(request): Json<PolicyStyxPrepareRequest>,
) -> Result<(), StatusCode> {
    let session_id = request.session_id;

    log::info!("Preparing computation for session {}", session_id);

    // Get a mutable lock on the sessions
    let mut sessions = sessions.lock().await;

    // --- Step 1: Check preconditions ---
    // First, check if the session even exists and if an app is already loaded.
    // If the app is already loaded, we can return an error immediately.
    if let Some(session) = sessions.get(&session_id) {
        // Use immutable .get() for the check
        if session.app_idx.is_some() {
            log::error!("Session {} already has an application loaded", session_id);
            return Err(StatusCode::BAD_REQUEST);
        }
    } else {
        // Or if the session doesn't exist at all
        log::error!("Session {} not found", session_id);
        return Err(StatusCode::NOT_FOUND); // Or appropriate error
    }

    // --- Step 2: Perform the expensive operation ---
    // Now that we've let go of any borrows from the check above, we can freely
    // create a new mutable borrow for `load_new_application`.
    let data_path = format!("./data/data-{session_id:?}");
    let program_path = format!("./data/program-{session_id:?}");

    let app_idx = sessions
        .rt // This creates a mutable borrow that ends right after this line.
        .load_new_application(&program_path)
        .map_err(|e| {
            log::error!(
                "Failed to load application for session {}: {}",
                session_id,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Load the data.
    let input_data = bincode::deserialize_from(
        fs::File::open(data_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sessions
        .rt
        .pcd_dataset_add_data(&session_id, input_data)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // --- Step 3: Update the session state ---
    // The borrow for `load_new_application` is now finished. We can start a new
    // borrow to update the session. We can use .unwrap() because we already
    // confirmed the session exists.
    sessions
        .get_mut(&session_id)
        .ok_or(StatusCode::NOT_FOUND)?
        .app_idx
        .replace(app_idx);

    Ok(())
}

async fn policy_styx_remote_attestation(
    State(sessions): State<Sessions>,
    Json(request): Json<PolicyStyxAttestationRequest>,
) -> Result<PolicyStyxAttestationResponse, StatusCode> {
    log::info!("Received remote attestation request");

    let gx = PublicKey::from_sec1_bytes(&request.gx).map_err(|_| StatusCode::BAD_REQUEST)?;

    #[cfg(not(feature = "mock"))]
    let report = attestation::get_tdx_attestation_report_raw()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    #[cfg(feature = "mock")]
    let report = vec![];

    let server_private_key = EphemeralSecret::random(&mut OsRng);

    let gy = server_private_key.public_key().to_sec1_bytes().to_vec();

    let mut sessions = sessions.lock().await;
    let session_id = Uuid::new_v4();

    sessions.insert(
        session_id,
        Session {
            id: session_id,
            app_idx: None,
            key: server_private_key
                .diffie_hellman(&gx)
                .raw_secret_bytes()
                .to_vec(), // The session key.
        },
    );

    Ok(PolicyStyxAttestationResponse {
        quote: report,
        quote_type: 1, // Assuming 1 is the type of the quote.
        gy,
        session_id,
    })
}

async fn policy_styx_upload(
    State(sessions): State<Sessions>,
    mut request: Multipart,
) -> Result<PolicyStyxUploadResponse, StatusCode> {
    log::info!("Starting file upload");
    
    let session_id = request
        .next_field()
        .await
        .map_err(|e| {
            log::error!("Failed to read sessionId field: {}", e);
            StatusCode::BAD_REQUEST
        })?;
    let session_id = match session_id {
        Some(field) => field.text().await.map_err(|e| {
            log::error!("Failed to parse sessionId as text: {}", e);
            StatusCode::BAD_REQUEST
        })?,
        None => {
            log::error!("sessionId field is missing");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    log::info!("Received sessionId: {}", session_id);

    // Decode session as base64 into a UUID.
    let session_id = match Uuid::parse_str(&session_id) {
        Ok(id) => id,
        Err(e) => {
            log::error!("Failed to parse sessionId as UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let sessions = sessions.lock().await;
    if let Some(session) = sessions.get(&session_id) {
        log::info!("Found session {}", session.id);

        let data = request
            .next_field()
            .await
            .map_err(|e| {
                log::error!("Failed to read data field: {}", e);
                StatusCode::BAD_REQUEST
            })?
            .ok_or_else(|| {
                log::error!("Data field is missing");
                StatusCode::BAD_REQUEST
            })?
            .bytes()
            .await
            .map_err(|e| {
                log::error!("Failed to read data bytes: {}", e);
                StatusCode::BAD_REQUEST
            })?;
        
        log::info!("Received data file ({} bytes)", data.len());

        let program = request
            .next_field()
            .await
            .map_err(|e| {
                log::error!("Failed to read program field: {}", e);
                StatusCode::BAD_REQUEST
            })?
            .ok_or_else(|| {
                log::error!("Program field is missing");
                StatusCode::BAD_REQUEST
            })?
            .bytes()
            .await
            .map_err(|e| {
                log::error!("Failed to read program bytes: {}", e);
                StatusCode::BAD_REQUEST
            })?;
        
        log::info!("Received program file ({} bytes)", program.len());

        let policy_engine = request
            .next_field()
            .await
            .map_err(|e| {
                log::error!("Failed to read policyEngine field: {}", e);
                StatusCode::BAD_REQUEST
            })?
            .ok_or_else(|| {
                log::error!("PolicyEngine field is missing");
                StatusCode::BAD_REQUEST
            })?
            .bytes()
            .await
            .map_err(|e| {
                log::error!("Failed to read policyEngine bytes: {}", e);
                StatusCode::BAD_REQUEST
            })?;
        
        log::info!("Received policy engine file ({} bytes)", policy_engine.len());

        fs::write(format!("./data/data-{session_id:?}"), &data)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        fs::write(format!("./data/program-{session_id:?}"), &program)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        fs::write(
            format!("./data/policy_engine-{session_id:?}"),
            &policy_engine,
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(PolicyStyxUploadResponse {})
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn policy_styx_compute(
    State(sessions): State<Sessions>,
    Json(request): Json<PolicyStyxComputeRequest>,
) -> Result<Vec<u8>, StatusCode> {
    log::info!(
        "Received compute request for session {}",
        request.session_id
    );

    let mut sessions = sessions.lock().await;
    if let Some(session) = sessions.get(&request.session_id) {
        // Here you would handle the computation using the session key.
        // For now, we just log it.
        log::info!("Processing compute for session {}", session.id);

        let idx = session.app_idx.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // Let the module knows its sesssion.
        let ptr = sessions
            .rt
            .write_memory(Some(idx), request.session_id.as_bytes())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        sessions
            .rt
            .execute_typed_function::<PcdWasmRawPtr, ()>(idx, &request.entry, ptr.into())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let args =
            bincode::serialize(&request.args).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let ptr = sessions
            .rt
            .write_memory(Some(idx), &args)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let ret = sessions
            .rt
            .read_memory(Some(idx), ptr)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(ret)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn serve(addr: &str, port: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", addr, port);
    // Define the CORS policy.
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods(cors::Any)
        .allow_headers(cors::Any);

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let app = Router::new()
        .route("/api/v1/upload", post(policy_styx_upload))
        .route("/api/v1/attestation", post(policy_styx_remote_attestation))
        .route("/api/v1/prepare", post(policy_styx_prepare_computation))
        .route("/api/v1/compute", post(policy_styx_compute))
        .layer(cors)
        .with_state(Arc::new(Mutex::new(ServerState::new().unwrap())));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
