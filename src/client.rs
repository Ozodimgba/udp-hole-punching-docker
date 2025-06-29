use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{io, thread};

use crate::logger::{ConnectionState, NatConsoleLogger, NatLoggable};
use crate::protocol::Message;

pub struct Client {
    id: String,
    socket: Arc<UdpSocket>, // share with background thread
    server_addr: SocketAddr,
    pub external_addr: Option<SocketAddr>,
    listening: bool,
    pub should_listen: Arc<AtomicBool>,
    pub connected_peers: Arc<std::sync::Mutex<std::collections::HashMap<String, SocketAddr>>>, // prolly shit but will do, refactor
    pub console_logger: NatConsoleLogger,
}

impl Client {
    pub fn new(id: String, server_addr: SocketAddr) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;
        println!(
            "🔌 Client '{}' created, local: {}",
            id,
            socket.local_addr()?
        );

        let socket = Arc::new(socket);
        let local_addr = socket.local_addr()?;
        let mut console_logger = NatConsoleLogger::new(local_addr);

        console_logger.print_address_table();

        let client = Self {
            id: id.clone(),
            socket: socket.clone(),
            server_addr,
            external_addr: None,
            listening: false,
            should_listen: Arc::new(AtomicBool::new(true)),
            connected_peers: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            console_logger,
        };

        Ok(client)
    }

    pub fn register(&mut self) -> io::Result<()> {
        let local_port = self.socket.local_addr()?.port();
        let msg = Message::Register {
            id: self.id.clone(),
            port: local_port,
        };
        self.send_to_server(&msg)?;
        println!("✅ Registration packet sent successfully");

        let mut buf = [0; 1024];
        let (len, _) = self.socket.recv_from(&mut buf)?;
        let response = Message::decode(&String::from_utf8_lossy(&buf[..len]))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if let Message::RegisterOk { external_addr } = response {
            self.external_addr = Some(external_addr);
            self.console_logger.set_external_addr(external_addr);
            println!("✅ Registered! External address: {}", external_addr);

            self.console_logger.print_address_table();
            // start background listening after successful registration
            self.start_background_listening()?;

            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Registration failed"))
        }
    }

    pub fn connect_to_peer(&mut self, peer_id: &str) -> io::Result<SocketAddr> {
        println!("🔍 Step 1: Discovering peer '{}'...", peer_id);

        // send discovery request
        let discover_msg = Message::Discover {
            target: peer_id.to_string(),
        };
        self.send_to_server(&discover_msg)?;

        // wait for discovery response (background listener will show it)
        thread::sleep(Duration::from_millis(1000));

        println!("🔍 Step 2: Requesting hole punch coordination...");
        let punch_msg = Message::HolePunch {
            from: self.id.clone(),
            to: peer_id.to_string(),
        };
        self.send_to_server(&punch_msg)?;
        println!("✅ Step 2: Hole punch request sent");

        println!("🔍 Step 3: Waiting for hole punch coordination...");
        println!("   (Background listener will handle START_PEER message)");

        // wait longer for hole punch to complete
        thread::sleep(Duration::from_millis(8000));

        self.print_status_report();

        // check if connection was established by looking at connected peers
        if let Ok(peers) = self.connected_peers.lock() {
            if let Some(&peer_addr) = peers.get("peer") {
                println!(
                    "✅ Hole punch successful! Connection established to {}",
                    peer_addr
                );
                return Ok(peer_addr);
            }
        }

        // fallback: return the discovered address even if hole punch uncertain
        println!("⚠️ Hole punch status uncertain, but proceeding...");

        // just return a dummy address - the actual connection is tracked in connected_peers
        Ok("127.0.0.1:1234".parse().unwrap())
    }

    // refator: auto-triggers connection for the receiving peer
    fn start_background_listening(&mut self) -> io::Result<()> {
        if self.listening {
            return Ok(());
        }

        self.listening = true;
        let socket = self.socket.clone();
        let client_id = self.id.clone();
        let should_listen = self.should_listen.clone();
        let connected_peers = self.connected_peers.clone();

        let mut bg_logger = NatConsoleLogger::new(self.socket.local_addr()?);
        if let Some(ext_addr) = self.external_addr {
            bg_logger.set_external_addr(ext_addr);
        }

        thread::spawn(move || {
            println!("🔊 Background listener started for {}", client_id);
            let mut buf = [0; 1024];

            while should_listen.load(Ordering::Relaxed) {
                match socket.recv_from(&mut buf) {
                    Ok((len, sender)) => {
                        let data = String::from_utf8_lossy(&buf[..len]);
                        println!(
                            "\n🔍 [{}] DEBUG: Received {} bytes from {}: '{}'",
                            client_id, len, sender, data
                        );

                        // Check if this is from the signaling server
                        if sender.port() == 9090 {
                            println!("📡 [{}] This is from signaling server", client_id);

                            match Message::decode(&data) {
                                Ok(msg) => {
                                    println!(
                                        "✅ [{}] Successfully parsed message: {:?}",
                                        client_id, msg
                                    );
                                    match msg {
                                        Message::StartPunchWithPeer {
                                            timestamp,
                                            peer_addr,
                                        } => {
                                            println!(
                                                "\n🚀 [{}] HOLE PUNCH COORDINATION RECEIVED!",
                                                client_id
                                            );
                                            println!(
                                                "🎯 [{}] Target: {}, Timestamp: {}",
                                                client_id, peer_addr, timestamp
                                            );

                                            bg_logger.log_peer_discovery(
                                                "peer".to_string(),
                                                Some(peer_addr),
                                            );

                                            // calculate delay
                                            let now = SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap()
                                                .as_millis()
                                                as u64;
                                            println!(
                                                "⏰ [{}] Now: {}, Start: {}",
                                                client_id, now, timestamp
                                            );

                                            if timestamp > now {
                                                let delay = timestamp - now;
                                                println!(
                                                    "⏳ [{}] Waiting {} ms before starting...",
                                                    client_id, delay
                                                );
                                                thread::sleep(Duration::from_millis(delay));
                                            }

                                            println!(
                                                "🕳️ [{}] STARTING HOLE PUNCH SEQUENCE TO {}",
                                                client_id, peer_addr
                                            );

                                            for i in 0..10 {
                                                bg_logger.log_hole_punch_attempt("peer");

                                                let punch_msg = format!("PUNCH:{}", i);
                                                match socket
                                                    .send_to(punch_msg.as_bytes(), peer_addr)
                                                {
                                                    Ok(_) => {
                                                        println!(
                                                            "🕳️ [{}] Sent hole punch {} to {}",
                                                            client_id, i, peer_addr
                                                        );
                                                    }
                                                    Err(e) => {
                                                        bg_logger.log_hole_punch_failure("peer");
                                                        println!(
                                                            "❌ [{}] Hole punch {} failed: {}",
                                                            client_id, i, e
                                                        );
                                                    }
                                                }
                                                thread::sleep(Duration::from_millis(50));
                                            }

                                            if let Ok(mut peers) = connected_peers.lock() {
                                                peers.insert("peer".to_string(), peer_addr);
                                                // println!("🔗 [{}] AUTO-CONNECTED: Stored connection to {}", client_id, peer_addr);
                                            }

                                            println!(
                                                "✅ [{}] Hole punch sequence completed to {}",
                                                client_id, peer_addr
                                            );
                                            println!(
                                                "🎉 [{}] Ready to send/receive messages!",
                                                client_id
                                            );
                                        }

                                        Message::StartPunch { timestamp } => {
                                            println!("🚀 [{}] Received OLD FORMAT hole punch (no peer address)", client_id);
                                        }

                                        Message::PeerFound { id, addr } => {
                                            bg_logger.log_peer_discovery(id.clone(), Some(addr));

                                            println!(
                                                "🔍 [{}] Peer discovery result: {} at {}",
                                                client_id, id, addr
                                            );
                                        }

                                        _ => {
                                            println!(
                                                "🔍 [{}] Other server message: {:?}",
                                                client_id, msg
                                            );
                                        }
                                    }
                                }
                                Err(parse_error) => {
                                    println!(
                                        "❌ [{}] Failed to parse server message '{}': {}",
                                        client_id, data, parse_error
                                    );
                                    println!("🔍 [{}] Raw bytes: {:?}", client_id, &buf[..len]);
                                }
                            }
                        } else {
                            // handle P2P messages (not from signaling server)
                            println!("🤝 [{}] This is P2P traffic from {}", client_id, sender);

                            if data.starts_with("MSG:") {
                                let message = &data[4..];
                                println!(
                                    "\n📥 [{}] Received message from {}: {}",
                                    client_id, sender, message
                                );
                                bg_logger.log_direct_message_received("peer", message, sender);
                                bg_logger.print_live_update("peer");
                            } else if data.starts_with("PUNCH:") {
                                println!(
                                    "\n🕳️ [{}] Received hole punch from {}: {}",
                                    client_id, sender, data
                                );
                                let response = format!("PUNCH_ACK:{}", client_id);
                                match socket.send_to(response.as_bytes(), sender) {
                                    Ok(_) => {
                                        println!("🤝 [{}] Sent punch ACK to {}", client_id, sender);

                                        let latency_ms = 50; // Placeholder - you could implement proper timing
                                        bg_logger.log_hole_punch_success("peer", latency_ms);
                                    }
                                    Err(e) => println!(
                                        "❌ [{}] Failed to send punch ACK: {}",
                                        client_id, e
                                    ),
                                }

                                // store the peer connection when receiving hole punch
                                if let Ok(mut peers) = connected_peers.lock() {
                                    peers.insert("peer".to_string(), sender);
                                    println!(
                                        "🔗 [{}] PUNCH-CONNECTED: Stored connection to {}",
                                        client_id, sender
                                    );
                                }
                            } else if data.starts_with("PUNCH_ACK:") {
                                println!(
                                    "\n🤝 [{}] Received punch ACK from {}: {}",
                                    client_id, sender, data
                                );

                                let latency_ms = 100; // Placeholder
                                bg_logger.log_hole_punch_success("peer", latency_ms);

                                // store the peer connection when receiving punch ACK
                                if let Ok(mut peers) = connected_peers.lock() {
                                    peers.insert("peer".to_string(), sender);
                                    println!(
                                        "🔗 [{}] ACK-CONNECTED: Stored connection to {}",
                                        client_id, sender
                                    );
                                }

                                println!("🎉 DIRECT P2P CONNECTION ESTABLISHED!");
                                bg_logger.print_live_update("peer");
                            } else {
                                println!(
                                    "\n🔍 [{}] Unknown P2P message from {}: {}",
                                    client_id, sender, data
                                );
                            }
                        }

                        print!("\n{} > ", client_id);
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        println!("❌ [{}] Background listener error: {}", client_id, e);
                        break;
                    }
                }
            }

            println!("🔇 Background listener stopped for {}", client_id);
        });

        Ok(())
    }

    pub fn send_message(&mut self, peer_addr: SocketAddr, message: &str) -> io::Result<()> {
        let data = format!("MSG:{}", message);
        self.socket.send_to(data.as_bytes(), peer_addr)?;

        self.log_message_sent("peer", message);
        self.console_logger.print_live_update("peer");

        println!("📤 Sent message to {}: {}", peer_addr, message);
        Ok(())
    }

    pub fn listen_for_messages(&self) -> io::Result<()> {
        // method is now optional since I add background listening
        // keep it for compatibility tho
        println!("Already listening in background. Messages will appear automatically.");
        Ok(())
    }

    fn send_to_server(&self, msg: &Message) -> io::Result<()> {
        let data = msg.encode();
        self.socket.send_to(data.as_bytes(), self.server_addr)?;
        Ok(())
    }

    pub fn get_connected_peers(&self) -> Vec<SocketAddr> {
        if let Ok(peers) = self.connected_peers.lock() {
            peers.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn has_connections(&self) -> bool {
        if let Ok(peers) = self.connected_peers.lock() {
            !peers.is_empty()
        } else {
            false
        }
    }

    pub fn print_detailed_report(&self) {
        let separator = "=".repeat(80);
        println!("\n{}", separator);
        println!("                    NAT TRAVERSAL DETAILED REPORT");
        println!("{}", separator);
        self.console_logger.print_full_report();
        println!("{}", separator);
    }
}

impl NatLoggable for Client {
    fn get_console_logger(&mut self) -> &mut NatConsoleLogger {
        &mut self.console_logger
    }

    fn get_console_logger_ref(&self) -> &NatConsoleLogger {
        &self.console_logger
    }
}
