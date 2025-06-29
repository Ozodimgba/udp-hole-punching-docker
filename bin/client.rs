use nat_traversal::client::Client;
use nat_traversal::logger::NatLoggable;
use std::env;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 NAT Traversal P2P Client with Logging");
    println!("==================================================");

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

    // Create and register client
    let mut client = Client::new(client_id.clone(), server_addr)?;

    println!("\n📡 Registering with signaling server...");
    client.register()?;

    println!("\n✅ Registration complete!");
    println!();
    print_commands();

    let mut connected_peer: Option<SocketAddr> = None;

    // Interactive command loop
    loop {
        print!("\n{} > ", client_id);
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
                println!("\n🔗 Initiating connection to peer '{}'...", peer_id);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                match client.connect_to_peer(peer_id) {
                    Ok(peer_addr) => {
                        connected_peer = Some(peer_addr);
                        println!("\n🎉 Connection process completed!");
                        println!("✅ You can now send messages to '{}'", peer_id);

                        // Print detailed report
                        client.print_detailed_report();
                    }
                    Err(e) => {
                        println!("❌ Connection failed: {}", e);
                        client.print_detailed_report();
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
                        Ok(()) => {
                            println!("✅ Message sent successfully!");
                        }
                        Err(e) => println!("❌ Send failed: {}", e),
                    }
                } else {
                    println!("❌ Not connected to any peer. Use 'connect <peer_id>' first.");
                }
            }

            "status" => {
                println!("\n📊 Current Client Status");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━");
                client.print_status_report();
            }

            "report" => {
                println!("\n📈 Detailed NAT Traversal Report");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                client.print_detailed_report();
            }

            "test" => {
                if parts.len() < 2 {
                    println!("❌ Usage: test <peer_id>");
                    continue;
                }

                let peer_id = parts[1];
                println!(
                    "\n🧪 Starting automated connection test to '{}'...",
                    peer_id
                );

                // Automated test sequence
                run_connection_test(&mut client, peer_id);
            }

            "monitor" => {
                println!("\n📡 Starting live monitoring mode...");
                println!("Press Ctrl+C to stop monitoring");
                start_live_monitoring(&client);
            }

            "help" => {
                print_commands();
            }

            "quit" | "exit" => {
                println!("\n📋 Final Report");
                println!("━━━━━━━━━━━━━━━");
                client.print_detailed_report();
                println!("\n👋 Goodbye!");
                break;
            }

            _ => {
                println!(
                    "❌ Unknown command: '{}'. Type 'help' for available commands.",
                    command
                );
            }
        }
    }

    Ok(())
}

fn print_commands() {
    println!("📚 Available Commands:");
    println!("━━━━━━━━━━━━━━━━━━━━━");
    println!("  connect <peer_id>  - Initiate hole punching with peer");
    println!("  send <message>     - Send direct P2P message");
    println!("  status            - Show current connection status");
    println!("  report            - Display detailed NAT traversal report");
    println!("  test <peer_id>     - Run automated connection test");
    println!("  monitor           - Start live connection monitoring");
    println!("  help              - Show this help message");
    println!("  quit              - Exit with final report");
}

fn run_connection_test(client: &mut Client, peer_id: &str) {
    println!("🔬 Test Phase 1: Connection Attempt");
    println!("───────────────────────────────────");

    match client.connect_to_peer(peer_id) {
        Ok(peer_addr) => {
            println!("✅ Phase 1 Complete: Connection established");

            println!("\n🔬 Test Phase 2: Message Exchange");
            println!("─────────────────────────────────────");

            // Send test messages
            for i in 1..=3 {
                let test_msg = format!("Test message #{} from automated test", i);
                match client.send_message(peer_addr, &test_msg) {
                    Ok(()) => println!("✅ Test message {} sent", i),
                    Err(e) => println!("❌ Test message {} failed: {}", i, e),
                }
                thread::sleep(Duration::from_millis(500));
            }

            println!("\n🔬 Test Phase 3: Final Results");
            println!("──────────────────────────────");
            client.print_detailed_report();
        }
        Err(e) => {
            println!("❌ Phase 1 Failed: {}", e);
            client.print_detailed_report();
        }
    }
}

fn start_live_monitoring(client: &Client) {
    println!("Starting live updates every 2 seconds...");

    for i in 1..=10 {
        thread::sleep(Duration::from_secs(2));
        println!("\n📊 Live Update #{}", i);
        println!("──────────────────");
        client.print_status_report();

        if i == 10 {
            println!("🏁 Live monitoring complete (10 updates)");
            break;
        }
    }
}
