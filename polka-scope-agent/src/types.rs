use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

/// Estados do agente - EXATAMENTE como na especificação TLA+
/// Ver: polka_scope.tla, linha 17
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    FetchingPeers,
    Probing,
    Reporting,
}

/// Representa um peer da rede P2P
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub peer_id: String,
    pub ip: IpAddr,
    pub port: u16,
}

/// Resultado de uma sondagem de latência
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub peer: Peer,
    pub latency_ms: u64,
    pub success: bool,
    pub timestamp: u64,
}

/// Resposta do RPC system_peers
#[derive(Debug, Deserialize)]
pub struct SystemPeersResponse {
    pub jsonrpc: String,
    pub result: Vec<PeerInfo>,
}

#[derive(Debug, Deserialize)]
pub struct PeerInfo {
    #[serde(rename = "peerId")]
    pub peer_id: String,
    pub roles: String,
    #[serde(rename = "bestHash")]
    pub best_hash: String,
    #[serde(rename = "bestNumber")]
    pub best_number: u64,
}

/// Configuração do agente
#[derive(Debug, Clone)]
pub struct Config {
    /// URL do RPC local (ex: http://localhost:9933)
    pub rpc_url: String,
    
    /// Porta P2P para sondar (ex: 30333)
    pub p2p_port: u16,
    
    /// Intervalo entre sondagens (segundos)
    pub probe_interval: Duration,
    
    /// URL do backend para enviar dados
    pub backend_url: Option<String>,
    
    /// Timeout para sondagem TCP (ms)
    pub probe_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:9933".to_string(),
            p2p_port: 30333,
            probe_interval: Duration::from_secs(30),
            backend_url: None,
            probe_timeout: Duration::from_millis(5000),
        }
    }
}