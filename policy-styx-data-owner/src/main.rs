mod qvl;

const PORT: &str = "10087"; // Port exposed to the outside.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]";

    // Start the QVL server.
    qvl::serve(addr, PORT).await
}
