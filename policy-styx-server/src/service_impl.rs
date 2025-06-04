use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use p256::ecdh::EphemeralSecret;
use p256::elliptic_curve::rand_core::OsRng;
use p256::PublicKey;
use policy_styx_lib::attestation;
use policy_styx_proto::policy_styx_service::policy_styx_service_server::PolicyStyxService;
use policy_styx_proto::policy_styx_service::{
    PolicyStyxAttestationRequest, PolicyStyxAttestationResponse, PolicyStyxUploadRequest,
};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

fn get_next_session_id(sessions: &HashMap<usize, Session>) -> usize {
    const COUNTER: AtomicUsize = AtomicUsize::new(0);

    // Generate a new session ID that is unique.
    let mut session_id = COUNTER.fetch_add(1, Ordering::SeqCst);
    while sessions.contains_key(&session_id) {
        session_id = COUNTER.fetch_add(1, Ordering::SeqCst);
    }
    session_id
}

/// A simple session.
#[derive(Debug, Default)]
struct Session {
    id: usize,
    key: Vec<u8>,
}

/// The server should be something that holds a mutable state because we have to
/// serve sessions with possibly different clients. For example, we will need to
/// store the ECDH session key used to protect the sensitive data.
#[derive(Debug, Default)]
pub struct PolicyStyxServer {
    // Maintains a session list.
    sessions: Arc<Mutex<HashMap<usize, Session>>>,
}

#[tonic::async_trait]
impl PolicyStyxService for PolicyStyxServer {
    async fn policy_styx_remote_attestation(
        &self,
        request: Request<PolicyStyxAttestationRequest>,
    ) -> Result<Response<PolicyStyxAttestationResponse>, Status> {
        // Extract the gx (the client's public key) from the request.
        let gx = PublicKey::from_sec1_bytes(&request.into_inner().gx)
            .map_err(|e| Status::invalid_argument(format!("Invalid gx public key: {}", e)))?;

        let report = attestation::get_tdx_attestation_report_raw().map_err(|e| {
            Status::internal(format!("Failed to get TDX attestation report: {}", e))
        })?;

        let server_private_key = EphemeralSecret::random(&mut OsRng);

        let gy = server_private_key.public_key().to_sec1_bytes().to_vec();
        let mut response = PolicyStyxAttestationResponse::default();
        response.quote = report;
        response.quote_type = 1;
        response.gy = gy;

        let mut sessions = self.sessions.lock().await;
        let session = get_next_session_id(&sessions);

        sessions.insert(
            session,
            Session {
                id: session,
                key: server_private_key
                    .diffie_hellman(&gx)
                    .raw_secret_bytes()
                    .to_vec(), // The session key.
            },
        );

        Ok(Response::new(response))
    }

    async fn policy_styx_upload(
        &self,
        request: Request<PolicyStyxUploadRequest>,
    ) -> Result<Response<()>, Status> {
        let upload_request = request.into_inner();
        let session_id = upload_request.session_id;

        let sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get(&(session_id as _)) {
            // Here you would handle the upload using the session key.
            // For now, we just log it.
            log::info!(
                "Received upload for session {} with key: {:?}",
                session.id,
                session.key
            );
            Ok(Response::new(()))
        } else {
            Err(Status::not_found(format!(
                "Session {} not found",
                session_id
            )))
        }
    }
}
