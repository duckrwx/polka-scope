use crate::types::{AgentStatus, Config, Peer, ProbeResult};
use crate::rpc_client::RpcClient;
use crate::prober::Prober;
use crate::reporter::Reporter;
use std::time::Instant;
use tracing::{info, error, debug};

/// Agente P2P principal
/// Implementa a máquina de estados validada em TLA+
pub struct P2PAgent {
    status: AgentStatus,
    config: Config,
    rpc_client: RpcClient,
    prober: Prober,
    reporter: Option<Reporter>,
    last_fetch_time: Instant,
    known_peers: Vec<Peer>,
    probe_results: Vec<ProbeResult>,
}

impl P2PAgent {
    pub fn new(config: Config) -> Self {
        let rpc_client = RpcClient::new(config.rpc_url.clone());
        let prober = Prober::new(config.p2p_port, config.probe_timeout);
        let reporter = config.backend_url.as_ref().map(|url| Reporter::new(url.clone()));
        
        Self {
            status: AgentStatus::Idle,
            config,
            rpc_client,
            prober,
            reporter,
            last_fetch_time: Instant::now(),
            known_peers: Vec::new(),
            probe_results: Vec::new(),
        }
    }
    
    /// Loop principal do agente
    /// Ver: polka_scope.tla, linha 75 (AgentLoop)
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🚀 Polka-Scope Agent iniciado");
        
        loop {
            // === ESTADO: IDLE ===
            // Ver: polka_scope.tla, linha 79
            self.idle().await?;
            
            // === ESTADO: FETCHING ===
            // Ver: polka_scope.tla, linha 88
            if let Err(e) = self.fetch_peers().await {
                error!("❌ Erro ao buscar peers: {}", e);
                // Em caso de erro, volta ao Idle (como na spec TLA+)
                self.status = AgentStatus::Idle;
                continue;
            }
            
            // === ESTADO: PROBING ===
            // Ver: polka_scope.tla, linha 115
            self.probe_peers().await?;
            
            // === ESTADO: REPORTING ===
            // Ver: polka_scope.tla, linha 141
            self.report_data().await?;
        }
    }
    
    /// Estado: Idle
    /// Aguarda o timer disparar
    async fn idle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.status = AgentStatus::Idle;
        debug!("💤 Estado: Idle");
        
        // Aguarda até que o intervalo tenha passado
        let elapsed = self.last_fetch_time.elapsed();
        if elapsed < self.config.probe_interval {
            let remaining = self.config.probe_interval - elapsed;
            info!("⏰ Aguardando {:?} até próxima sondagem", remaining);
            tokio::time::sleep(remaining).await;
        }
        
        Ok(())
    }
    
    /// Estado: FetchingPeers
    /// Busca lista de peers via RPC
    async fn fetch_peers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.status = AgentStatus::FetchingPeers;
        info!("🔍 Estado: Buscando peers...");
        
        self.last_fetch_time = Instant::now();
        
        // Chama system_peers via RPC
        self.known_peers = self.rpc_client.get_peers().await?;
        
        info!("✅ Encontrados {} peers", self.known_peers.len());
        
        Ok(())
    }
    
    /// Estado: Probing
    /// Sonda cada peer para medir latência
    async fn probe_peers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.status = AgentStatus::Probing;
        info!("📡 Estado: Sondando peers...");
        
        self.probe_results.clear();
        
        // Ver: polka_scope.tla, linha 119 (loop sobre peers)
        for peer in &self.known_peers {
            debug!("🔌 Sondando peer: {}", peer.peer_id);
            
            let result = self.prober.probe(peer).await;
            
            match result.success {
                true => info!("  ✓ {} -> {}ms", peer.ip, result.latency_ms),
                false => info!("  ✗ {} -> timeout", peer.ip),
            }
            
            self.probe_results.push(result);
        }
        
        info!("✅ Sondagem completa: {}/{} peers alcançados",
              self.probe_results.iter().filter(|r| r.success).count(),
              self.probe_results.len());
        
        Ok(())
    }
    
    /// Estado: Reporting
    /// Envia dados para o dashboard
    async fn report_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.status = AgentStatus::Reporting;
        info!("📤 Estado: Enviando dados...");
        
        if let Some(reporter) = &self.reporter {
            reporter.send_results(&self.probe_results).await?;
            info!("✅ Dados enviados ao backend");
        } else {
            // Modo sem backend - apenas exibe no console
            info!("📊 Resultados:");
            for result in &self.probe_results {
                println!("  {} -> {}ms ({})",
                         result.peer.ip,
                         result.latency_ms,
                         if result.success { "OK" } else { "TIMEOUT" });
            }
        }
        
        Ok(())
    }
    
    /// Retorna o status atual (para debugging)
    pub fn status(&self) -> &AgentStatus {
        &self.status
    }
}

// Implementação de Debug invariantes
// Reflete o TypeInvariant da especificação TLA+
impl P2PAgent {
    /// Verifica invariantes do sistema
    /// Ver: polka_scope.tla, linha 40 (TypeInvariant)
    pub fn check_invariants(&self) -> bool {
        // Status deve ser válido
        matches!(
            self.status,
            AgentStatus::Idle
                | AgentStatus::FetchingPeers
                | AgentStatus::Probing
                | AgentStatus::Reporting
        )
    }
}