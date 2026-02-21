use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use log::{info, warn};

#[derive(Serialize)]
pub struct ClassifyRequest {
    pub images: Vec<String>, // Absolute paths
}

#[derive(Deserialize, Debug)]
pub struct ClassifyResult {
    pub path: String,
    pub category: String,
    pub confidence: f32,
}

#[derive(Deserialize, Debug)]
pub struct BatchClassifyResponse {
    pub results: Vec<ClassifyResult>,
}

#[derive(Deserialize)]
struct HealthResponse {
    status: String,
}

pub async fn warm_up(client: &Client, url: &str) -> anyhow::Result<()> {
    info!("Waiting for AI microservice to be ready at {}/health...", url);
    let max_retries = 30;
    for attempt in 1..=max_retries {
        match client.get(&format!("{}/health", url)).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(body) = resp.json::<HealthResponse>().await {
                    if body.status == "ready" {
                        info!("AI microservice is ready.");
                        return Ok(());
                    }
                    warn!("Microservice loading (status: {}), retry {}/{}...", body.status, attempt, max_retries);
                }
            }
            _ => {
                warn!("Microservice not reachable, retry {}/{}...", attempt, max_retries);
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    anyhow::bail!("AI microservice did not become ready after {} retries ({}s)", max_retries, max_retries * 2);
}

pub async fn classify_batch(client: &Client, url: &str, paths: Vec<String>) -> anyhow::Result<Vec<ClassifyResult>> {
    let req = ClassifyRequest { images: paths };
    let resp = client.post(&format!("{}/classify/batch", url))
        .json(&req)
        .send()
        .await?;
        
    let batch_resp: BatchClassifyResponse = resp.json().await?;
    Ok(batch_resp.results)
}
