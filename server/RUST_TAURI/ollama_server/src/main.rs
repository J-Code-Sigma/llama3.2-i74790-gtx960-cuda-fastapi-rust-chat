use actix_web::{post, web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;
use chrono::Local;
use reqwest::Client;
use tokio::time::{sleep, Duration};

#[derive(Deserialize, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatRequest {
    prompt: String,
    messages: Option<Vec<ChatMessage>>,
}

#[derive(Serialize)]
struct OllamaResponse {
    response: String,
}

#[post("/v1/chat")]
async fn chat(
    body: web::Json<ChatRequest>,
    semaphore: web::Data<Arc<Semaphore>>,
    client: web::Data<Client>,
) -> impl Responder {
    let _permit = semaphore.acquire().await.unwrap();
    let start_time = Local::now();
    println!("[{}] Received prompt: {}", start_time.format("%H:%M:%S"), body.prompt);

    // Ollama configuration from env or default
    let ollama_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://ollama:11434".to_string());
    let ollama_model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());

    let topics = std::fs::read_to_string("topics.txt").unwrap_or_else(|_| "General assistance".to_string());
    let system_prompt = format!("You are a specialized AI assistant. You stay strictly on these topics: {}. If a user asks about other topics, you MUST state that you do not have access and cannot help with those. NEVER pretend to have information outside these topics. You also DO NOT HAVE ACCESS to user accounts, passwords, or personal data.", topics);

    let mut payload = serde_json::json!({
        "model": ollama_model,
        "prompt": body.prompt,
        "system": system_prompt,
        "stream": false
    });

    if let Some(messages) = &body.messages {
        payload["messages"] = serde_json::json!(messages);
    }
    println!("[{}] Sending to Ollama: {}", start_time.format("%H:%M:%S"), payload);

    let resp_result = client
        .post(format!("{}/api/generate", ollama_host))
        .json(&payload)
        .send()
        .await;

    let response_text = match resp_result {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => json["response"].as_str().unwrap_or("No response field found").to_string(),
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
