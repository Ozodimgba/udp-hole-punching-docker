use nat_traversal::server::Server;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Simple NAT Traversal - Signaling Server");
    println!("==========================================");

    // get bind address
    let bind_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:9090".to_string());

    println!("Starting signaling server on {}", bind_addr);

    let mut server = Server::new(&bind_addr)?;

    println!("✅ Signaling server ready!");
    println!("   Clients can register and discover peers");
    println!("   Press Ctrl+C to stop");
    println!();

    server.run()?;

    Ok(())
}
