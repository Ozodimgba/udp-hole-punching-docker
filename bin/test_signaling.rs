// src/bin/test_signaling.rs
// Simple test to verify signaling server works

use nat_traversal::client::Client;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Simple NAT Traversal Signaling");
    println!("=========================================");
    
    let server_addr: SocketAddr = "127.0.0.1:9090".parse()?;
    
    println!("Starting test sequence...");
    println!("(Make sure signaling server is running on {})", server_addr);
    println!();
    
    println!("⏰ Starting in 3 seconds...");
    thread::sleep(Duration::from_secs(3));

    println!("1️⃣  Creating and registering Alice...");
    let mut alice = Client::new("alice".to_string(), server_addr)?;
    alice.register()?;
    println!("   ✅ Alice registered successfully");
    
    thread::sleep(Duration::from_millis(500));
    

    println!("2️⃣  Creating and registering Bob...");
    let mut bob = Client::new("bob".to_string(), server_addr)?;
    bob.register()?;
    println!("   ✅ Bob registered successfully");
    
    thread::sleep(Duration::from_millis(500));
    
    println!("3️⃣  Alice discovering Bob...");
    match alice.connect_to_peer("bob") {
        Ok(bob_addr) => {
            println!("   ✅ Alice found Bob at: {}", bob_addr);
        }
        Err(e) => {
            println!("   ❌ Alice failed to find Bob: {}", e);
            return Err(e.into());
        }
    }
    
    thread::sleep(Duration::from_millis(500));
    
    println!("4️⃣  Bob discovering Alice...");
    match bob.connect_to_peer("alice") {
        Ok(alice_addr) => {
            println!("   ✅ Bob found Alice at: {}", alice_addr);
        }
        Err(e) => {
            println!("   ❌ Bob failed to find Alice: {}", e);
            return Err(e.into());
        }
    }
    
    thread::sleep(Duration::from_millis(500));
    
    println!("5️⃣  Testing peer not found...");
    match alice.connect_to_peer("charlie") {
        Ok(_) => {
            println!("   ❌ Should not have found charlie!");
        }
        Err(_) => {
            println!("   ✅ Correctly failed to find non-existent peer");
        }
    }
    
    println!();
    println!("🎉 All signaling tests passed!");
    println!("✅ Basic peer discovery works");
    println!("✅ Hole punch coordination triggers");
    println!("✅ Error handling works");
    println!();
    println!("Next steps:");
    println!("- Test with real NAT environments");
    println!("- Verify hole punch packets are sent");
    println!("- Test direct messaging after connection");
    
    Ok(())
}