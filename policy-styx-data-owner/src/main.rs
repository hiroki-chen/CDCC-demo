use ecdsa::signature::rand_core::OsRng;
use p256::ecdh::EphemeralSecret;
use policy_styx_proto::policy_styx_service::policy_styx_service_client::PolicyStyxServiceClient;
use policy_styx_proto::policy_styx_service::PolicyStyxAttestationRequest;
use tonic::Request;

mod qvl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PolicyStyxServiceClient::connect("http://127.0.0.1:10086").await?;

    let secret_key = EphemeralSecret::random(&mut OsRng);
    let public_key = secret_key.public_key();

    let attestation_request = Request::new(PolicyStyxAttestationRequest {
        // Fill in the request details as needed.
        gx: public_key.to_sec1_bytes().to_vec(),
    });

    let response = client
        .policy_styx_remote_attestation(attestation_request)
        .await?;
    let response = response.into_inner();

    // We first verify the quote.
    qvl::verify_quote(&response.quote)?;

    // We then extract the public key.
    let server_public_key = p256::PublicKey::from_sec1_bytes(&response.gy)
        .map_err(|e| format!("Failed to parse server public key: {}", e))?;

    let session_key = secret_key.diffie_hellman(&server_public_key);

    Ok(())
}
