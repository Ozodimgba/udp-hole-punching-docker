use crate::protocol::Message;
use std::net::{UdpSocket, SocketAddr};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io;

pub struct Server {
    socket: UdpSocket,
    clients: HashMap<String, SocketAddr>,
}

impl Server {
    pub fn new(addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        println!("📡 Server listening on {}", socket.local_addr()?);
        Ok(Self { socket, clients: HashMap::new() })
    }
    
    pub fn run(&mut self) -> io::Result<()> {
        let mut buf = [0; 1024];
        
        loop {
            let (len, client_addr) = self.socket.recv_from(&mut buf)?;
            let msg_str = String::from_utf8_lossy(&buf[..len]);
            
            match Message::decode(&msg_str) {
                Ok(msg) => self.handle_message(msg, client_addr)?,
                Err(e) => println!("❌ Parse error: {}", e),
            }
        }
    }
    
    fn handle_message(&mut self, msg: Message, addr: SocketAddr) -> io::Result<()> {
        match msg {
            Message::Register { id, port } => {
                let external_addr = SocketAddr::new(addr.ip(), port);
                self.clients.insert(id.clone(), external_addr);
                let response = Message::RegisterOk { external_addr };
                self.send_to(&response, addr)?;
                println!("✅ Registered {} at {}", id, external_addr);
            }
            Message::Discover { target } => {
                if let Some(&peer_addr) = self.clients.get(&target) {
                    let response = Message::PeerFound { id: target, addr: peer_addr };
                    self.send_to(&response, addr)?;
                } else {
                    let response = Message::PeerNotFound { id: target };
                    self.send_to(&response, addr)?;
                }
            }
            Message::HolePunch { from, to } => {
                if let (Some(&from_addr), Some(&to_addr)) = (self.clients.get(&from), self.clients.get(&to)) {
                    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 + 2000;

                    let start_msg_to_requester = Message::StartPunchWithPeer { 
                        timestamp, 
                        peer_addr: to_addr 
                    };
                    self.send_to(&start_msg_to_requester, from_addr)?;
                    
                    let start_msg_to_target = Message::StartPunchWithPeer { 
                        timestamp, 
                        peer_addr: from_addr 
                    };
                    self.send_to(&start_msg_to_target, to_addr)?;
                    
                    println!("🕳️  Coordinating hole punch: {} ({}) ↔ {} ({})", from, from_addr, to, to_addr);
                } else {
                    println!("❌ Cannot coordinate hole punch: missing client addresses");
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn send_to(&self, msg: &Message, addr: SocketAddr) -> io::Result<()> {
        let data = msg.encode();
        self.socket.send_to(data.as_bytes(), addr)?;
        println!("📤 Sent to {}: {}", addr, data);
        Ok(())
    }
}