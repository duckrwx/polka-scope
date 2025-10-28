use crate::types::{Peer, ProbeResult};
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

pub struct Prober {
    port: u16,
    timeout_duration: Duration,
}

impl Prober {
    pub fn new(port: u16, timeout_duration: Duration) -> Self {
        Self {
            port,
            timeout_duration,
        }
    }
    
    /// Sonda um peer medindo latência do handshake TCP
    pub async fn probe(&self, peer: &Peer) -> ProbeResult {
        let addr = SocketAddr::new(peer.ip, self.port);
        let start = Instant::now();
        
        let success = match timeout(self.timeout_duration, TcpStream::connect(addr)).await {
            Ok(Ok(_stream)) => {
                // Conexão estabelecida com sucesso
                true
            }
            Ok(Err(_)) | Err(_) => {
                // Erro de conexão ou timeout
                false
            }
        };
        
        let latency_ms = if success {
            start.elapsed().as_millis() as u64
        } else {
            999 // Timeout (como na spec TLA+)
        };
        
        ProbeResult {
            peer: peer.clone(),
            latency_ms,
            success,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}