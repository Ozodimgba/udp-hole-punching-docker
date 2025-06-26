use nat_traversal::client::Client;
use std::env;
use std::io::{self, Write};
use std::net::SocketAddr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Simple NAT Traversal - P2P Client");
    println!("====================================");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <client_id> [server_addr]", args[0]);
        println!("Example: {} alice", args[0]);
        println!("Example: {} bob 192.168.1.100:9090", args[0]);
        return Ok(());
    }

    let client_id = args[1].clone();
    let server_addr = args
        .get(2)
        .unwrap_or(&"127.0.0.1:9090".to_string())
        .parse::<SocketAddr>()?;

    println!("Client ID: {}", client_id);
    println!("Server: {}", server_addr);
    println!();

    // Create and register client
    let mut client = Client::new(client_id.clone(), server_addr)?;

    println!("📡 Registering with signaling server...");
    client.register()?;

    println!();
    println!("✅ Registration complete!");
    println!("Commands:");
    println!("  connect <peer_id>  - Connect to another peer");
    println!("  send <message>     - Send message to connected peer");
    println!("  listen            - Listen for incoming messages");
    println!("  status            - Show client status");
    println!("  quit              - Exit");
    println!();

    let mut connected_peer: Option<SocketAddr> = None;

    // Interactive command loop
    loop {
        print!("{} > ", client_id);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];

        match command {
            "connect" => {
                if parts.len() < 2 {
                    println!("❌ Usage: connect <peer_id>");
                    continue;
                }

                let peer_id = parts[1];
                println!("Connecting to peer '{}'...", peer_id);

                match client.connect_to_peer(peer_id) {
                    Ok(peer_addr) => {
                        connected_peer = Some(peer_addr);
                        println!("✅ Connected to '{}' at {}", peer_id, peer_addr);
                        println!("   You can now send messages!");
                    }
                    Err(e) => {
                        println!("❌ Connection failed: {}", e);
                    }
                }
            }

            "send" => {
                if parts.len() < 2 {
                    println!("❌ Usage: send <message>");
                    continue;
                }

                if let Some(peer_addr) = connected_peer {
                    let message = parts[1..].join(" ");
                    match client.send_message(peer_addr, &message) {
                        Ok(()) => println!("✅ Message sent!"),
                        Err(e) => println!("❌ Send failed: {}", e),
                    }
                } else {
                    println!("❌ Not connected to any peer. Use 'connect <peer_id>' first.");
                }
            }

            "listen" => {
                println!("ℹ️ Already listening in background automatically!");
                println!("   Messages will appear as they arrive.");
            }

            "status" => {
                println!("📊 Client Status:");
                println!("   ID: {}", client_id);
                println!("   External Address: {:?}", client.external_addr);
                println!("   Connected Peer: {:?}", connected_peer);
            }

            "help" => {
                println!("📚 Available commands:");
                println!("   connect <peer_id>  - Connect to another peer");
                println!("   send <message>     - Send message to connected peer");
                println!("   listen            - Listen for incoming messages");
                println!("   status            - Show client status");
                println!("   help              - Show this help");
                println!("   quit              - Exit");
            }

            "quit" | "exit" => {
                println!("👋 Goodbye!");
                break;
            }

            _ => {
                println!(
                    "❌ Unknown command: {}. Type 'help' for available commands.",
                    command
                );
            }
        }

        println!();
    }

    Ok(())
}
