use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use p256::ecdh::EphemeralSecret;
use p256::elliptic_curve::rand_core::OsRng;
use p256::PublicKey;
#[cfg(all(not(feature = "mock"), feature = "platform-tdx"))]
use policy_styx_lib::attestation;
use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;
use tokio::sync::Mutex;
use tower_http::cors::{self, AllowOrigin, CorsLayer};
use uuid::Uuid;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PolicyStyxAttestationRequest {
    // base64.
    #[serde_as(as = "Base64")]
    gx: Vec<u8>, // The client's public key.
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PolicyStyxAttestationResponse {
    #[serde_as(as = "Base64")]
    gy: Vec<u8>, // The server's public key.
    #[serde_as(as = "Base64")]
    quote: Vec<u8>, // The attestation report.
    quote_type: u32, // The type of the quote.
    #[serde_as(as = "Base64")]
    session_id: Uuid, // The session ID.
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PolicyStyxUploadRequest {
    #[serde_as(as = "Base64")]
    session_id: Uuid, // The session ID for the upload.
    #[serde_as(as = "Base64")]
    data: Vec<u8>, // The data to be uploaded.
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

/// A simple session.
#[derive(Debug, Default)]
pub struct Session {
    id: Uuid,
    key: Vec<u8>,
}

type Sessions = Arc<Mutex<HashMap<Uuid, Session>>>;

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
    Json(request): Json<PolicyStyxUploadRequest>,
) -> Result<PolicyStyxUploadResponse, StatusCode> {
    let session_id = request.session_id;

    let sessions = sessions.lock().await;
    if let Some(session) = sessions.get(&session_id) {
        // Here you would handle the upload using the session key.
        // For now, we just log it.
        log::info!(
            "Received upload for session {} with key: {:?}",
            session.id,
            session.key
        );

        Ok(PolicyStyxUploadResponse {})
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
        .layer(cors)
        .with_state(Sessions::default());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
