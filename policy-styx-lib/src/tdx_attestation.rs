use anyhow::{anyhow, Result};
#[cfg(feature = "platform-tdx")]
use dcap_rs::types::quotes::version_4::QuoteV4;
#[cfg(feature = "platform-tdx")]
use tdx::*;

#[cfg(feature = "platform-tdx")]
#[inline]
/// Fetch the TDX attestation report from `/dev/tdx-guest`.
///
/// # Note
///
/// This is guest-side function which means this function will directly retrieve the
/// report from the special device created by the tdx module for the patched guest
/// kernel. Calling this function on the host will result in an error.
pub fn get_tdx_attestation_report() -> Result<QuoteV4> {
    let uid = unsafe { libc::getuid() };
    // Make sure we are root.
    if uid != 0 {
        return Err(anyhow!("Must run in root! Your user id is {uid}"));
    }

    // Talk with the TDX host to fetch its report.
    let tdx = Tdx::new();

    tdx.get_attestation_report()
        .map_err(|e| anyhow!("Cannot obtain the report"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(feature = "platform-tdx")]
    #[test]
    fn test_get_tdx_attestation_report() {
        let report = get_tdx_attestation_report();

        assert!(
            report.is_ok(),
            "Failed to fetch the report: {}",
            report.err().unwrap()
        );
    }
}
