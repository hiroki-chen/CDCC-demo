use anyhow::Result;

const PORT: &str = "10086"; // Port exposed to the outside.

mod service_impl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "mock"))]
    sudo::escalate_if_needed()?;

    crate::service_impl::serve("[::]", PORT).await
}
