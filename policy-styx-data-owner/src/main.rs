use anyhow::{bail, Context, Result};
use dcap_rs::types::quotes::body::{EnclaveReport, TD10ReportBody};
use dcap_rs::types::quotes::version_4::{QuoteSignatureDataV4, QuoteV4};
use dcap_rs::types::quotes::{CertData, QuoteHeader};
use ecdsa::signature::Verifier;
use ecdsa::VerifyingKey;
use json::JsonValue;
use p256::ecdsa::Signature;
use p256::NistP256;
use sha2::{Digest, Sha256};
use ureq::tls::TlsConfig;
use ureq::Agent;
use x509_certificate::X509Certificate;

const URL: &str = "https://localhost:8081/sgx/certification/v4";
/// This is our FMSPC fetched from the pckcert.
///
/// If you are curious about how to get this value, please refer to
///     `GET https://api.trustedservices.intel.com/sgx/certification/v4/pckcert`
///     `-H "Ocp-Apim-Subscription-Key: xxx"`
/// whose arguments can be found in `pckidretrievaltool` tool. You will need an
/// active subscription to that service.
const FMSPC: &str = "90C06F000000";

/// Fetch the collateral from the PCS to verify the quote. Note we will need a valid
/// FMSPC *(Family, Model,. Stepping, Platform Type, and Customized SKU)
///
/// This function is primarily for TCB infor validation; not the first target.
#[allow(unused)]
fn get_collateral(fmspc: &str) -> Result<JsonValue> {
    let url = format!("{URL}/tcb?fmspc={fmspc}");

    let agent: Agent = Agent::config_builder()
        .tls_config(TlsConfig::builder().disable_verification(true).build())
        .build()
        .into();

    let res = agent.get(&url).call()?.body_mut().read_to_vec()?;
    let res = String::from_utf8(res)?;

    json::parse(&res).context("cannot parse json")
}

/// Verify if the ceritifcate carried by the attestation key is valid.
fn verify_quoting_enclave(signature: &QuoteSignatureDataV4, cert_data: &CertData) -> Result<()> {
    // First let's check the certificate data type.
    match cert_data.cert_data_type {
        // According to https://download.01.org/intel-sgx/latest/dcap-latest/linux/docs/Intel_TDX_DCAP_Quoting_Library_API.pdf#page=39.44,
        // type 6 indicates that this is a QE certification data.
        //
        // QE Report Certification Data = QE Report | QE Report Signature | QE Authentication Data | QE Certification Data | PCK Cert Chain
        6 => {
            let cert_data = &cert_data.cert_data;
            // SGX report of the Quoting Enclave that generated an Attestation key.
            let qe_report_raw = &cert_data[..384];
            // report data is SHA256(ECDSA Key) || QE Authentication Data || 32 0x00s
            let qe_report = EnclaveReport::from_bytes(&qe_report_raw);
            // ECDSA signature over the QE report calculated using the PCK.
            let qe_report_signature = Signature::from_bytes((&cert_data[384..384 + 64]).into())?;
            // QE Authentication data.
            let qe_authentication_data_len =
                u16::from_le_bytes((&cert_data[384 + 64..384 + 64 + 2]).try_into()?);
            let qe_authentication_data =
                &cert_data[384 + 64 + 2..384 + 64 + 2 + qe_authentication_data_len as usize];
            let qe_certification_data =
                &cert_data[384 + 64 + 2 + qe_authentication_data_len as usize..];

            if qe_certification_data[0] != 5 {
                bail!("unsupported QE certification data");
            }

            // Extract the certificate chain of PCK LEAF || Intermeadite CA || Root CA
            // They are PEM encoded.
            let concat_certs = &qe_certification_data[6..];
            let certs = X509Certificate::from_pem_multiple(&concat_certs)
                .context("Failed to parse certificates")?;

            if certs.len() != 3 {
                bail!("The certificate chain is ill-formed. Should be three!");
            }

            let pck = &certs[0];
            let intermediate = &certs[1];
            let root_ca = &certs[2];

            // TODO: verify the certificate chain.

            // Check if QE is signed.
            let verifying_key =
                VerifyingKey::<NistP256>::from_sec1_bytes(&pck.public_key_data().to_vec())?;

            verifying_key.verify(qe_report_raw, &qe_report_signature)?;

            let msg = {
                let mut msg = vec![];
                msg.extend_from_slice(&signature.ecdsa_attestation_key);
                msg.extend_from_slice(&qe_authentication_data);
                msg
            };
            let key_hash = Sha256::digest(&msg).to_vec();
            let expected_key_hash = &qe_report.report_data[..32];

            if &key_hash != expected_key_hash {
                println!("The key hash does not match!\n\tExpecting {expected_key_hash:?}, got {key_hash:?}");
            }

            Ok(())
        },
        ty => bail!("unsupported certificate date: {ty}"),
    }
}

fn verify_quote_signature(raw_quote: &[u8]) -> Result<()> {
    const HEADER_SIZE: usize = std::mem::size_of::<QuoteHeader>();
    const BODY_SIZE: usize = std::mem::size_of::<TD10ReportBody>();

    let quote = QuoteV4::from_bytes(raw_quote);

    if quote.header.version < 3 {
        bail!("Not a tdx quote!");
    }

    // The cert chain is already embedded into the quote so there is no need to check again the collateral.
    // Or you can obtain it via the PCK API to get "SGX-PCK-Certificate-Issuer-Chain".
    // The collateral contains only the TCB information.

    let signature = &quote.signature;

    // Verify if the certificate chain is valid.
    verify_quoting_enclave(signature, &signature.qe_cert_data)?;

    // Verify if the ECDSA signature is valid.
    let raw_message = &raw_quote[..HEADER_SIZE + BODY_SIZE];
    let raw_signature = Signature::from_bytes(&signature.quote_signature.into())?;
    let raw_key = {
        let mut raw_key = Vec::with_capacity(65);
        raw_key.push(0x04);
        raw_key.extend_from_slice(&signature.ecdsa_attestation_key);
        raw_key
    };

    let verifying_key = VerifyingKey::<NistP256>::from_sec1_bytes(&raw_key)?;

    verifying_key
        .verify(raw_message, &raw_signature)
        .context("Failed to verify the signature")
}

fn main() {
    // TODO: Should first talk to the server (we should use gRPC for request handling).
    let quote = std::fs::read("data/quote.bin").unwrap();

    verify_quote_signature(&quote).unwrap();
}
