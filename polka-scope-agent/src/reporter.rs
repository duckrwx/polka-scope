use crate::types::ProbeResult;
use reqwest::Client;
use serde_json::json;
use tracing::debug;

pub struct Reporter {
    backend_url: String,
    client: Client,
}

impl Reporter {
    pub fn new(backend_url: String) -> Self {
        Self {
            backend_url,
            client: Client::new(),
        }
    }
    
    /// Envia resultados para o backend
    pub async fn send_results(&self, results: &[ProbeResult]) -> Result<(), Box<dyn std::error::Error>> {
        debug!("📤 Enviando {} resultados para backend", results.len());
        
        let payload = json!({
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "results": results
        });
        
        let response = self.client
            .post(&self.backend_url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Backend error: {}", response.status()).into());
        }
        
        Ok(())
    }
}