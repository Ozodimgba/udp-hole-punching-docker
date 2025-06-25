use std::net::SocketAddr;

// ============================================================================
// SIMPLE PROTOCOL
// ============================================================================

#[derive(Debug, Clone)]
pub enum Message {
    Register { id: String, port: u16 },
    RegisterOk { external_addr: SocketAddr },
    Discover { target: String },
    PeerFound { id: String, addr: SocketAddr },
    PeerNotFound { id: String },
    HolePunch { from: String, to: String },
    StartPunch { timestamp: u64 },
}

impl Message {
    pub fn encode(&self) -> String {
        match self {
            Message::Register { id, port } => format!("REG:{}:{}", id, port),
            Message::RegisterOk { external_addr } => format!("OK:{}", external_addr),
            Message::Discover { target } => format!("FIND:{}", target),
            Message::PeerFound { id, addr } => format!("PEER:{}:{}", id, addr),
            Message::PeerNotFound { id } => format!("NOPE:{}", id),
            Message::HolePunch { from, to } => format!("PUNCH:{}:{}", from, to),
            Message::StartPunch { timestamp } => format!("START:{}", timestamp),
        }
    }
    
    pub fn decode(s: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 2 {
            return Err("Invalid message format");
        }
        
        match parts[0] {
            "REG" if parts.len() == 3 => {
                Ok(Message::Register { 
                    id: parts[1].to_string(), 
                    port: parts[2].parse().map_err(|_| "Invalid port")? 
                })
            },
            "OK" if parts.len() >= 2 => {
                // IPv4 addresses 
                let addr_str = if parts.len() == 3 {
                    format!("{}:{}", parts[1], parts[2])
                } else {
                    // IPv6 addresses
                    parts[1..].join(":")
                };
                Ok(Message::RegisterOk { 
                    external_addr: addr_str.parse().map_err(|_| "Invalid address")? 
                })
            },
            "FIND" => Ok(Message::Discover { target: parts[1].to_string() }),
            "PEER" if parts.len() >= 3 => {
                // handle both IPv4 and IPv6 addresses
                let addr_str = if parts.len() == 4 && !parts[2].starts_with('[') {
                    format!("{}:{}", parts[2], parts[3])
                } else {
                    // IPv6 or complex address: rejoin everything after peer ID
                    parts[2..].join(":")
                };
                Ok(Message::PeerFound { 
                    id: parts[1].to_string(), 
                    addr: addr_str.parse().map_err(|_| "Invalid peer address")? 
                })
            },
            "NOPE" => Ok(Message::PeerNotFound { id: parts[1].to_string() }),
            "PUNCH" if parts.len() == 3 => {
                Ok(Message::HolePunch { 
                    from: parts[1].to_string(), 
                    to: parts[2].to_string() 
                })
            },
            "START" => {
                Ok(Message::StartPunch { 
                    timestamp: parts[1].parse().map_err(|_| "Invalid timestamp")? 
                })
            },
            _ => Err("Unknown message type"),
        }
    }
}