use std::net::{UdpSocket, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{io, thread};

use crate::protocol::Message;

pub struct Client {
    id: String,
    socket: Arc<UdpSocket>, // share with background thread
    server_addr: SocketAddr,
    pub external_addr: Option<SocketAddr>,
    listening: bool,
}

impl Client {
    pub fn new(id: String, server_addr: SocketAddr) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;
        println!("🔌 Client '{}' created, local: {}", id, socket.local_addr()?);
        
        let socket = Arc::new(socket);
        
        let mut client = Self {
            id: id.clone(),
            socket: socket.clone(),
            server_addr,
            external_addr: None,
            listening: false,
        };
        
        Ok(client)
    }
    
    pub fn register(&mut self) -> io::Result<()> {
        let local_port = self.socket.local_addr()?.port();
        let msg = Message::Register { id: self.id.clone(), port: local_port };
        self.send_to_server(&msg)?;
        println!("✅ Registration packet sent successfully");
        
        let mut buf = [0; 1024];
        let (len, _) = self.socket.recv_from(&mut buf)?;
        let response = Message::decode(&String::from_utf8_lossy(&buf[..len]))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        if let Message::RegisterOk { external_addr } = response {
            self.external_addr = Some(external_addr);
            println!("✅ Registered! External address: {}", external_addr);

            // start background listening after successful registration
            self.start_background_listening()?;
        
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Registration failed"))
        }
    }
    
    pub fn connect_to_peer(&mut self, peer_id: &str) -> io::Result<SocketAddr> {
        println!("Debug: server_addr = {}", self.server_addr);
        
        println!("Temporarily stopping background listener...");
        self.listening = false;
        thread::sleep(Duration::from_millis(100)); 

        println!("Step 1: Discovering peer '{}'...", peer_id);
        let discover_msg = Message::Discover { target: peer_id.to_string() };
        self.send_to_server(&discover_msg)?;
        
        let mut buf = [0; 1024];
        let (len, _) = self.socket.recv_from(&mut buf)?;
        let response = Message::decode(&String::from_utf8_lossy(&buf[..len]))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let peer_addr = match response {
            Message::PeerFound { addr, .. } => {
                println!("✅ Step 1 complete: Found peer at {}", addr);
                addr
            },
            Message::PeerNotFound { .. } => return Err(io::Error::new(io::ErrorKind::NotFound, "Peer not found")),
            _ => return Err(io::Error::new(io::ErrorKind::Other, "Unexpected response")),
        };
        
        println!(" Found peer '{}' at {}", peer_id, peer_addr);
        
        println!("🔍 Step 2: Requesting hole punch coordination...");
        let punch_msg = Message::HolePunch { from: self.id.clone(), to: peer_id.to_string() };
        self.send_to_server(&punch_msg)?;
        println!("✅ Step 2: Hole punch request sent");

        println!("🔍 Step 3: Waiting for START signal...");
        let (len, _) = self.socket.recv_from(&mut buf)?;
        println!("✅ Step 3: Received {} bytes", len);
        
        let start_response = Message::decode(&String::from_utf8_lossy(&buf[..len]))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        if let Message::StartPunch { timestamp } = start_response {
            println!("🔍 Step 4: Executing hole punch...");
            self.execute_hole_punch(peer_addr, timestamp)?;
            println!("✅ Step 4: Hole punch complete");
        }
        
        Ok(peer_addr)
    }
    
    fn execute_hole_punch(&self, target_addr: SocketAddr, start_timestamp: u64) -> io::Result<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        if start_timestamp > now {
            thread::sleep(Duration::from_millis(start_timestamp - now));
        }
        
        // rapid fire 10 hole punch packets
        println!("🕳️  Starting hole punch to {}", target_addr);
        for i in 0..10 {
            let punch_data = format!("PUNCH:{}", i);
            self.socket.send_to(punch_data.as_bytes(), target_addr)?;
            thread::sleep(Duration::from_millis(50));
        }
        
        println!("✅ Hole punch sequence completed");
        Ok(())
    }

    fn start_background_listening(&mut self) -> io::Result<()> {
        if self.listening {
            return Ok(());
        }
        
        self.listening = true;
        let socket = self.socket.clone();
        let client_id = self.id.clone();
        
        thread::spawn(move || {
            println!("🔊 Background listener started for {}", client_id);
            let mut buf = [0; 1024];
            
            loop {
    match socket.recv_from(&mut buf) {
        Ok((len, sender)) => {
            let data = String::from_utf8_lossy(&buf[..len]);
            
            if sender.ip().to_string() == "192.168.1.2" && sender.port() == 9090 {
                println!("\n [{}] Server response intercepted by background listener: {}", client_id, data);
                println!("This response should be handled by main thread, not background listener");
                print!("{} > ", client_id);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            } else if data.starts_with("MSG:") {
                println!("\n [{}] Received from {}: {}", client_id, sender, &data[4..]);
                print!("{} > ", client_id); 
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            } else if data.starts_with("PUNCH:") {
                println!("[{}] Received hole punch from {}", client_id, sender);
                let _ = socket.send_to(b"PUNCH_ACK", sender);
                print!("{} > ", client_id);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            } else {
                // Unknown message type
                println!(" [{}] Unknown message from {}: {}", client_id, sender, data);
                print!("{} > ", client_id);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
            // no data available, continue listening
            thread::sleep(Duration::from_millis(10));
        }
        Err(_) => {
            break;
        }
    }
}
            
            println!("\n🔇 Background listener stopped for {}", client_id);
        });
        
        Ok(())
    }
    
    pub fn send_message(&self, peer_addr: SocketAddr, message: &str) -> io::Result<()> {
        let data = format!("MSG:{}", message);
        self.socket.send_to(data.as_bytes(), peer_addr)?;
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

    pub fn handle_incoming_signaling(&mut self) -> io::Result<bool> {
        let mut buf = [0; 1024];
        match self.socket.recv_from(&mut buf) {
            Ok((len, sender)) => {
                // check if this is from the signaling server
                if sender == self.server_addr {
                    let msg_str = String::from_utf8_lossy(&buf[..len]);
                    if let Ok(msg) = Message::decode(&msg_str) {
                        match msg {
                            Message::StartPunch { timestamp } => {
                                println!("🎯 Received hole punch coordination! Someone is connecting to me...");
                                println!("🔊 Automatically starting to listen for hole punch packets...");
                                
                                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                                if timestamp > now {
                                    std::thread::sleep(std::time::Duration::from_millis(timestamp - now));
                                }
                                
                                // Listen for hole punch packets for 10 seconds
                                for _ in 0..100 {
                                    self.listen_for_messages()?;
                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                }
                                
                                return Ok(true); // signal that we handled a coordination message
                            }
                            _ => {}
                        }
                    }
                }
                Ok(false)
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(false),
            Err(e) => Err(e),
        }
    }
}