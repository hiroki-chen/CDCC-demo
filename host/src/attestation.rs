use anyhow::Result;
#[cfg(feature = "platform-tdx")]
use dcap_rs::types::quotes::version_4::QuoteV4;

#[cfg(feature = "platform-tdx")]
#[inline]
/// Fetch the TDX attestation report from the TDX virtual machine.
/// 
/// # Note
/// 
/// This is the host-side function so basically it will create a new session to
/// talk with the TDX machine. This will eventually call *into* the VM.
pub fn get_tdx_attestation_report() -> Result<QuoteV4> {
    // Talk with the TDX host to fetch its report.
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(feature = "platform-tdx")]
    #[test]
    fn test_get_tdx_attestation_report() {
        assert!(get_tdx_attestation_report().is_ok())
    }
}
