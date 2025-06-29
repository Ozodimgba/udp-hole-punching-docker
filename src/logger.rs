use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct NatTraversalStats {
    pub local_addr: SocketAddr,
    pub external_addr: Option<SocketAddr>,
    pub peer_id: String,
    pub peer_addr: Option<SocketAddr>,
    pub hole_punch_attempts: u32,
    pub successful_hole_punches: u32,
    pub direct_messages_sent: u32,
    pub direct_messages_received: u32,
    pub traversal_success: bool,
    pub last_attempt_time: Option<Instant>,
    pub total_latency_ms: u64,
    pub connection_state: ConnectionState,
    pub error_count: u32,
}

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Discovering,
    HolePunching,
    Connected,
    Failed,
    Disconnected,
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionState::Discovering => write!(f, "DISC"),
            ConnectionState::HolePunching => write!(f, "PUNCH"),
            ConnectionState::Connected => write!(f, "CONN"),
            ConnectionState::Failed => write!(f, "FAIL"),
            ConnectionState::Disconnected => write!(f, "DISC"),
        }
    }
}

pub struct NatConsoleLogger {
    stats: HashMap<String, NatTraversalStats>,
    start_time: Instant,
    local_addr: SocketAddr,
    external_addr: Option<SocketAddr>,
}

impl NatConsoleLogger {
    pub fn new(local_addr: SocketAddr) -> Self {
        Self {
            stats: HashMap::new(),
            start_time: Instant::now(),
            local_addr,
            external_addr: None,
        }
    }

    pub fn set_external_addr(&mut self, external_addr: SocketAddr) {
        self.external_addr = Some(external_addr);
    }

    pub fn log_peer_discovery(&mut self, peer_id: String, peer_addr: Option<SocketAddr>) {
        let entry = self
            .stats
            .entry(peer_id.clone())
            .or_insert_with(|| NatTraversalStats {
                local_addr: self.local_addr,
                external_addr: self.external_addr,
                peer_id: peer_id.clone(),
                peer_addr,
                hole_punch_attempts: 0,
                successful_hole_punches: 0,
                direct_messages_sent: 0,
                direct_messages_received: 0,
                traversal_success: false,
                last_attempt_time: None,
                total_latency_ms: 0,
                connection_state: ConnectionState::Discovering,
                error_count: 0,
            });

        entry.peer_addr = peer_addr;
        entry.connection_state = ConnectionState::Discovering;

        println!("🔍 Discovered peer: {} at {:?}", peer_id, peer_addr);
    }

    pub fn log_hole_punch_attempt(&mut self, peer_id: &str) {
        if let Some(stats) = self.stats.get_mut(peer_id) {
            stats.hole_punch_attempts += 1;
            stats.last_attempt_time = Some(Instant::now());
            stats.connection_state = ConnectionState::HolePunching;
        }

        println!(
            "🕳️  Hole punch attempt #{} to peer: {}",
            self.stats
                .get(peer_id)
                .map(|s| s.hole_punch_attempts)
                .unwrap_or(0),
            peer_id
        );
    }

    pub fn log_hole_punch_success(&mut self, peer_id: &str, latency_ms: u64) {
        if let Some(stats) = self.stats.get_mut(peer_id) {
            stats.successful_hole_punches += 1;
            stats.traversal_success = true;
            stats.total_latency_ms += latency_ms;
            stats.connection_state = ConnectionState::Connected;
        }

        println!(
            "✅ Hole punch SUCCESS for peer: {} ({}ms)",
            peer_id, latency_ms
        );
    }

    pub fn log_hole_punch_failure(&mut self, peer_id: &str) {
        if let Some(stats) = self.stats.get_mut(peer_id) {
            stats.connection_state = ConnectionState::Failed;
            stats.error_count += 1;
        }

        println!("❌ Hole punch FAILED for peer: {}", peer_id);
    }

    pub fn log_direct_message_sent(&mut self, peer_id: &str, message: &str) {
        if let Some(stats) = self.stats.get_mut(peer_id) {
            stats.direct_messages_sent += 1;
        }

        println!("📤 Direct message sent to {}: {}", peer_id, message);
    }

    pub fn log_direct_message_received(
        &mut self,
        peer_id: &str,
        message: &str,
        sender_addr: SocketAddr,
    ) {
        if let Some(stats) = self.stats.get_mut(peer_id) {
            stats.direct_messages_received += 1;
        }

        println!(
            "📥 Direct message from {} ({}): {}",
            peer_id, sender_addr, message
        );
    }

    pub fn log_connection_failed(&mut self, peer_id: &str, error: &str) {
        if let Some(stats) = self.stats.get_mut(peer_id) {
            stats.connection_state = ConnectionState::Failed;
            stats.error_count += 1;
        }

        println!("🔥 Connection failed to {}: {}", peer_id, error);
    }

    // Print address information table
    pub fn print_address_table(&self) {
        println!("\n");
        self.print_section_header("Address Information");
        println!("┌─────────────────┬─────────────────────────────┐");
        println!("│ Address Type    │ Value                       │");
        println!("├─────────────────┼─────────────────────────────┤");
        println!("│ Local Address   │ {:<27} │", self.local_addr);

        match self.external_addr {
            Some(addr) => println!("│ External Address│ {:<27} │", addr),
            None => println!("│ External Address│ {:<27} │", "Not yet discovered"),
        }

        println!("└─────────────────┴─────────────────────────────┘");
    }

    // Print main NAT traversal results table
    pub fn print_traversal_table(&self) {
        if self.stats.is_empty() {
            return;
        }

        println!("\n");
        self.print_section_header(&format!(
            "NAT Traversal Results (peers: {})",
            self.stats.len()
        ));

        println!("┌────────────────┬──────────────────────┬─────────┬─────────┬─────────┬─────────┬─────────┬───────────┬─────────┐");
        println!(
            "│ {:<14} │ {:<20} │ {:<7} │ {:<7} │ {:<7} │ {:<7} │ {:<7} │ {:<9} │ {:<7} │",
            "Peer ID",
            "Peer Address",
            "State",
            "Hole",
            "Success",
            "Msg Out",
            "Msg In",
            "Avg RTT",
            "Errors"
        );
        println!(
            "│ {:<14} │ {:<20} │ {:<7} │ {:<7} │ {:<7} │ {:<7} │ {:<7} │ {:<9} │ {:<7} │",
            "", "", "", "Punches", "Rate", "", "", "", ""
        );
        println!("├────────────────┼──────────────────────┼─────────┼─────────┼─────────┼─────────┼─────────┼───────────┼─────────┤");

        let mut sorted_peers: Vec<_> = self.stats.iter().collect();
        sorted_peers.sort_by_key(|(peer_id, _)| peer_id.as_str());

        for (peer_id, stats) in sorted_peers {
            let peer_addr_str = stats
                .peer_addr
                .map(|addr| addr.to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let success_rate = if stats.hole_punch_attempts > 0 {
                format!(
                    "{:.1}%",
                    (stats.successful_hole_punches as f64 / stats.hole_punch_attempts as f64)
                        * 100.0
                )
            } else {
                "N/A".to_string()
            };

            let avg_rtt = if stats.successful_hole_punches > 0 {
                format!(
                    "{}ms",
                    stats.total_latency_ms / stats.successful_hole_punches as u64
                )
            } else {
                "N/A".to_string()
            };

            println!(
                "│ {:<14} │ {:<20} │ {:<7} │ {:<7} │ {:<7} │ {:<7} │ {:<7} │ {:<9} │ {:<7} │",
                pad_string(&truncate_string(peer_id, 14), 14),
                pad_string(&truncate_string(&peer_addr_str, 20), 20),
                pad_string(&format!("{}", stats.connection_state), 7),
                pad_string(&format!("{}", stats.hole_punch_attempts), 7),
                pad_string(&success_rate, 7),
                pad_string(&format!("{}", stats.direct_messages_sent), 7),
                pad_string(&format!("{}", stats.direct_messages_received), 7),
                pad_string(&avg_rtt, 9),
                pad_string(&format!("{}", stats.error_count), 7)
            );
        }

        println!("└────────────────┴──────────────────────┴─────────┴─────────┴─────────┴─────────┴─────────┴───────────┴─────────┘");

        // Summary statistics
        self.print_summary_stats();
    }

    // Print message log table for successful connections
    pub fn print_message_table(&self) {
        let connected_peers: Vec<_> = self
            .stats
            .iter()
            .filter(|(_, stats)| matches!(stats.connection_state, ConnectionState::Connected))
            .collect();

        if connected_peers.is_empty() {
            return;
        }

        println!("\n");
        self.print_section_header("Direct P2P Message Statistics");

        println!(
            "┌────────────────┬──────────────────────┬─────────────┬─────────────┬─────────────┐"
        );
        println!(
            "│ Peer ID        │ Peer Address         │ Msgs Sent  │ Msgs Recv  │ Total Msgs  │"
        );
        println!(
            "├────────────────┼──────────────────────┼─────────────┼─────────────┼─────────────┤"
        );

        for (peer_id, stats) in connected_peers {
            let peer_addr_str = stats
                .peer_addr
                .map(|addr| addr.to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let total_messages = stats.direct_messages_sent + stats.direct_messages_received;

            println!(
                "│ {:<14} │ {:<20} │ {:<11} │ {:<11} │ {:<11} │",
                truncate_string(peer_id, 14),
                truncate_string(&peer_addr_str, 20),
                stats.direct_messages_sent,
                stats.direct_messages_received,
                total_messages
            );
        }

        println!(
            "└────────────────┴──────────────────────┴─────────────┴─────────────┴─────────────┘"
        );
    }

    fn print_summary_stats(&self) {
        let total_peers = self.stats.len();
        let successful_connections = self
            .stats
            .values()
            .filter(|stats| stats.traversal_success)
            .count();
        let total_attempts = self
            .stats
            .values()
            .map(|stats| stats.hole_punch_attempts)
            .sum::<u32>();
        let total_successes = self
            .stats
            .values()
            .map(|stats| stats.successful_hole_punches)
            .sum::<u32>();
        let total_messages = self
            .stats
            .values()
            .map(|stats| stats.direct_messages_sent + stats.direct_messages_received)
            .sum::<u32>();

        let success_rate = if total_attempts > 0 {
            (total_successes as f64 / total_attempts as f64) * 100.0
        } else {
            0.0
        };

        let elapsed = self.start_time.elapsed();

        println!("┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐");
        println!("│ SUMMARY: {} total peers | {:.1}% hole punch success | {} total messages | {}ms elapsed │",
            total_peers,
            success_rate,
            total_messages,
            elapsed.as_millis()
        );
        println!("└─────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘");
    }

    fn print_section_header(&self, title: &str) {
        let border = "═".repeat(title.len() + 4);
        println!("┌{}┐", border);
        println!("│ {} │", title);
        println!("└{}┘", border);
    }

    // method to print all tables
    pub fn print_full_report(&self) {
        self.print_address_table();
        self.print_traversal_table();
        self.print_message_table();

        if self.stats.values().any(|s| s.traversal_success) {
            println!(
                "\n🎉 NAT Traversal successful for {} peer(s)!",
                self.stats.values().filter(|s| s.traversal_success).count()
            );
        } else {
            println!("\n❌ No successful NAT traversals yet. Check network configuration.");
        }
    }

    // Method to print live updates 
    pub fn print_live_update(&self, peer_id: &str) {
        if let Some(stats) = self.stats.get(peer_id) {
            let status_char = match stats.connection_state {
                ConnectionState::Discovering => "🔍",
                ConnectionState::HolePunching => "🕳️",
                ConnectionState::Connected => "✅",
                ConnectionState::Failed => "❌",
                ConnectionState::Disconnected => "🔌",
            };

            println!(
                "{} {} │ attempts: {} │ success: {} │ msgs: {}↑/{}↓ │",
                status_char,
                truncate_string(peer_id, 12),
                stats.hole_punch_attempts,
                stats.successful_hole_punches,
                stats.direct_messages_sent,
                stats.direct_messages_received
            );
        }
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn pad_string(s: &str, width: usize) -> String {
    if s.len() >= width {
        truncate_string(s, width)
    } else {
        format!("{:<width$}", s, width = width)
    }
}

pub trait NatLoggable {
    fn get_console_logger(&mut self) -> &mut NatConsoleLogger;
    fn get_console_logger_ref(&self) -> &NatConsoleLogger;

    fn log_peer_discovered(&mut self, peer_id: String, peer_addr: Option<SocketAddr>) {
        self.get_console_logger()
            .log_peer_discovery(peer_id, peer_addr);
    }

    fn log_hole_punch_attempt(&mut self, peer_id: &str) {
        self.get_console_logger().log_hole_punch_attempt(peer_id);
    }

    fn log_hole_punch_result(&mut self, peer_id: &str, success: bool, latency_ms: Option<u64>) {
        if success {
            if let Some(latency) = latency_ms {
                self.get_console_logger()
                    .log_hole_punch_success(peer_id, latency);
            }
        } else {
            self.get_console_logger().log_hole_punch_failure(peer_id);
        }
    }

    fn log_message_sent(&mut self, peer_id: &str, message: &str) {
        self.get_console_logger()
            .log_direct_message_sent(peer_id, message);
    }

    fn log_message_received(&mut self, peer_id: &str, message: &str, sender_addr: SocketAddr) {
        self.get_console_logger()
            .log_direct_message_received(peer_id, message, sender_addr);
    }

    fn print_status_report(&self) {
        self.get_console_logger_ref().print_full_report();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_console_logger_creation() {
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), 5000);
        let logger = NatConsoleLogger::new(local_addr);
        assert_eq!(logger.local_addr, local_addr);
    }

    #[test]
    fn test_peer_discovery_logging() {
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), 5000);
        let mut logger = NatConsoleLogger::new(local_addr);

        let peer_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 2, 10)), 5000);
        logger.log_peer_discovery("alice".to_string(), Some(peer_addr));

        assert!(logger.stats.contains_key("alice"));
        assert_eq!(logger.stats["alice"].peer_addr, Some(peer_addr));
    }

    #[test]
    fn test_hole_punch_tracking() {
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), 5000);
        let mut logger = NatConsoleLogger::new(local_addr);

        logger.log_peer_discovery("bob".to_string(), None);
        logger.log_hole_punch_attempt("bob");
        logger.log_hole_punch_success("bob", 150);

        let stats = &logger.stats["bob"];
        assert_eq!(stats.hole_punch_attempts, 1);
        assert_eq!(stats.successful_hole_punches, 1);
        assert_eq!(stats.total_latency_ms, 150);
        assert!(stats.traversal_success);
    }

    pub fn demo_traversal_tables() {
        println!("🚀 NAT Traversal Console Logger Demo");
        println!("=====================================\n");

        // create logger with dummy local address
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), 5000);
        let mut logger = NatConsoleLogger::new(local_addr);

        // set external address (discovered from signaling server)
        let external_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 45)), 5000);
        logger.set_external_addr(external_addr);

        // simulate discovering several peers
        println!("🔍 Simulating peer discovery...");

        // Bob - Successful connection
        let bob_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 2, 10)), 5000);
        logger.log_peer_discovery("bob".to_string(), Some(bob_addr));
        logger.log_hole_punch_attempt("bob");
        std::thread::sleep(std::time::Duration::from_millis(50)); // Simulate time
        logger.log_hole_punch_success("bob", 145);
        logger.log_direct_message_sent("bob", "Hello Bob!");
        logger.log_direct_message_received("bob", "Hi Alice!", bob_addr);
        logger.log_direct_message_sent("bob", "How's your NAT?");
        logger.log_direct_message_received("bob", "Working great!", bob_addr);

        // Charlie - failed after multiple attempts
        let charlie_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 3, 10)), 5000);
        logger.log_peer_discovery("charlie".to_string(), Some(charlie_addr));
        logger.log_hole_punch_attempt("charlie");
        std::thread::sleep(std::time::Duration::from_millis(30));
        logger.log_hole_punch_attempt("charlie");
        std::thread::sleep(std::time::Duration::from_millis(30));
        logger.log_hole_punch_attempt("charlie");
        logger.log_hole_punch_failure("charlie");

        // Dave - successful but high latency
        let dave_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 4, 15)), 5001);
        logger.log_peer_discovery("dave".to_string(), Some(dave_addr));
        logger.log_hole_punch_attempt("dave");
        logger.log_hole_punch_attempt("dave");
        std::thread::sleep(std::time::Duration::from_millis(40));
        logger.log_hole_punch_success("dave", 342);
        logger.log_direct_message_sent("dave", "Finally connected!");

        // Eve - trying to connect
        let eve_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 5, 20)), 5002);
        logger.log_peer_discovery("eve".to_string(), Some(eve_addr));
        logger.log_hole_punch_attempt("eve");
        logger.log_hole_punch_attempt("eve");
        // still in hole punching state

        // frank - Unknown address (signaling worked but no direct connection info)
        logger.log_peer_discovery("frank".to_string(), None);
        logger.log_hole_punch_attempt("frank");
        logger.log_hole_punch_failure("frank");

        // Grace - Very successful connection with lots of messages
        let grace_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 100)), 5000);
        logger.log_peer_discovery("grace".to_string(), Some(grace_addr));
        logger.log_hole_punch_attempt("grace");
        std::thread::sleep(std::time::Duration::from_millis(20));
        logger.log_hole_punch_success("grace", 89);

        // simulate a conversation
        for i in 1..=5 {
            logger.log_direct_message_sent("grace", &format!("Message {} to Grace", i));
            logger.log_direct_message_received("grace", &format!("Grace reply {}", i), grace_addr);
        }

        println!("\n✅ Demo data generated! Displaying tables...\n");

        logger.print_full_report();

        println!("\n📊 Live Updates Demo:");
        println!("=====================");
        logger.print_live_update("bob");
        logger.print_live_update("charlie");
        logger.print_live_update("dave");
        logger.print_live_update("eve");
        logger.print_live_update("frank");
        logger.print_live_update("grace");

        println!("\n🎉 Demo complete! This is what your NAT traversal logs will look like.");
    }

    #[test]
    fn run_demo() {
        demo_traversal_tables();
    }
}
