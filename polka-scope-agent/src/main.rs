mod agent;
mod rpc_client;
mod prober;
mod reporter;
mod types;

use agent::P2PAgent;
use clap::Parser;
use std::time::Duration;
use tracing_subscriber;
use types::Config;

#[derive(Parser, Debug)]
#[command(name = "polka-scope")]
#[command(about = "P2P monitoring agent for Substrate nodes", long_about = None)]
struct Args {
    /// URL do RPC do nó Substrate
    #[arg(short, long, default_value = "http://localhost:9933")]
    rpc_url: String,
    
    /// Porta P2P para sondar
    #[arg(short, long, default_value_t = 30333)]
    port: u16,
    
    /// Intervalo entre sondagens (segundos)
    #[arg(short, long, default_value_t = 30)]
    interval: u64,
    
    /// URL do backend (opcional)
    #[arg(short, long)]
    backend: Option<String>,
    
    /// Timeout para sondagem (ms)
    #[arg(short, long, default_value_t = 5000)]
    timeout: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializa logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();
    
    // Parse argumentos CLI
    let args = Args::parse();
    
    // Cria configuração
    let config = Config {
        rpc_url: args.rpc_url,
        p2p_port: args.port,
        probe_interval: Duration::from_secs(args.interval),
        backend_url: args.backend,
        probe_timeout: Duration::from_millis(args.timeout),
    };
    
    println!("╔════════════════════════════════════════╗");
    println!("║      POLKA-SCOPE P2P MONITOR          ║");
    println!("╚════════════════════════════════════════╝");
    println!();
    println!("📡 RPC URL: {}", config.rpc_url);
    println!("🔌 P2P Port: {}", config.p2p_port);
    println!("⏰ Interval: {:?}", config.probe_interval);
    if let Some(backend) = &config.backend_url {
        println!("📤 Backend: {}", backend);
    }
    println!();
    
    // Cria e executa agente
    let mut agent = P2PAgent::new(config);
    
    // Debug: verifica invariantes iniciais
    assert!(agent.check_invariants(), "Invariantes violadas no início!");
    
    agent.run().await?;
    
    Ok(())
}