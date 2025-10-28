use crate::types::{Peer, SystemPeersResponse};
use reqwest::Client;
use serde_json::json;
use std::net::IpAddr;
use tracing::{debug, error};

pub struct RpcClient {
    url: String,
    client: Client,
}

impl RpcClient {
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: Client::new(),
        }
    }
    
    /// Chama o método system_peers do Substrate
    pub async fn get_peers(&self) -> Result<Vec<Peer>, Box<dyn std::error::Error>> {
        debug!("📞 Chamando RPC: system_peers");
        
        let body = json!({
            "jsonrpc": "2.0",
            "method": "system_peers",
            "params": [],
            "id": 1
        });
        
        let response = self.client
            .post(&self.url)
            .json(&body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            error!("❌ RPC error: {}", response.status());
            return Err("RPC call failed".into());
        }
        
        let rpc_response: SystemPeersResponse = response.json().await?;
        
        // Extrai IPs dos peers
        let peers = rpc_response
            .result
            .into_iter()
            .filter_map(|peer_info| {
                // Tenta extrair IP do peer_id
                // No Substrate, peer_id geralmente contém o IP
                // Formato: /ip4/192.168.1.1/tcp/30333/p2p/12D3KooW...
                self.extract_ip_from_peer_id(&peer_info.peer_id)
                    .map(|ip| Peer {
                        peer_id: peer_info.peer_id,
                        ip,
                        port: 30333, // Porta P2P padrão
                    })
            })
            .collect();
        
        Ok(peers)
    }
    
    fn extract_ip_from_peer_id(&self, peer_id: &str) -> Option<IpAddr> {
        // Parse do multiaddr format: /ip4/192.168.1.1/tcp/30333/...
        for part in peer_id.split('/') {
            if let Ok(ip) = part.parse::<IpAddr>() {
                return Some(ip);
            }
        }
        None
    }
}