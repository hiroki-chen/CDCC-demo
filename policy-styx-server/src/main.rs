use anyhow::Result;
use policy_styx_proto::policy_styx_service::policy_styx_service_server::PolicyStyxServiceServer;
use tonic::transport::Server;

use crate::service_impl::PolicyStyxServer;

const PORT: &str = "10086"; // Port exposed to the outside.

mod service_impl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("[::1]:{}", PORT).parse()?;
    let service = PolicyStyxServer::default();

    env_logger::init();

    Server::builder()
        .add_service(PolicyStyxServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
