use actix_web::{post, web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;
use chrono::Local;
use reqwest::Client;
use tokio::time::{sleep, Duration};

#[derive(Deserialize)]
struct ChatRequest {
    prompt: String,
}

#[derive(Serialize)]
struct OllamaResponse {
    response: String,
}

#[post("/v1/run/tinyllama")]
async fn chat(
    body: web::Json<ChatRequest>,
    semaphore: web::Data<Arc<Semaphore>>,
    client: web::Data<Client>,
) -> impl Responder {
    let _permit = semaphore.acquire().await.unwrap();
    let start_time = Local::now();
    println!("[{}] Received prompt: {}", start_time.format("%H:%M:%S"), body.prompt);

    // Ollama host from env or default
    let ollama_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://ollama:11434".to_string());

    let resp_result = client
        .post(format!("{}/api/generate", ollama_host))
        .json(&serde_json::json!({
            "model": "tinyllama",
            "prompt": body.prompt,
            "stream": false
        }))
        .send()
        .await;

    let response_text = match resp_result {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => json.to_string(),
            Err(_) => "Invalid response from Ollama".to_string(),
        },
        Err(_) => "Failed to contact Ollama".to_string(),
    };

    let end_time = Local::now();
    println!(
        "[{}] Finished request ({} -> {})",
        end_time.format("%H:%M:%S"),
        start_time.format("%H:%M:%S"),
        end_time.format("%H:%M:%S")
    );

    HttpResponse::Ok().json(OllamaResponse { response: response_text })
}

async fn wait_for_ollama(ollama_host: &str, client: &Client) {
    loop {
        match client.get(format!("{}/api/tags", ollama_host)).send().await {
            Ok(resp) if resp.status().is_success() => break,
            _ => {
                println!("Waiting for Ollama to be ready...");
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let semaphore = Arc::new(Semaphore::new(1));
    let client = Client::new();
    let ollama_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://ollama:11434".to_string());

    // Wait for Ollama before starting the server
    wait_for_ollama(&ollama_host, &client).await;

    println!("Starting Rust API on port 8080...");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(semaphore.clone()))
            .app_data(web::Data::new(client.clone()))
            .service(chat)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
