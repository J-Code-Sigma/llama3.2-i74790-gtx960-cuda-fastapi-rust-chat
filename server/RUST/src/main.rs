use actix_web::{post, web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;
use chrono::Local;
use reqwest::Client;
use tokio::time::{sleep, Duration};

#[derive(Deserialize, Serialize, Clone)]
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
struct llamacppResponse {
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

    // Llamacpp configuration from env or default
    let llammacpp_host = std::env::var("LLAMACPP_HOST").unwrap_or_else(|_| "http://llamacpp:11434".to_string());
    let llammacpp_model = std::env::var("LLAMACPP_MODEL").unwrap_or_else(|_| "llama3.2".to_string());

    let topics = std::fs::read_to_string("topics.txt").unwrap_or_else(|_| "General assistance".to_string());

    let system_prompt;
    if(topics.is_empty()) {
        system_prompt = "You are a specialized AI assistant. You stay strictly on these topics: General assistance. If a user asks about other topics, you MUST state that you do not have access and cannot help with those. You also DO NOT HAVE ACCESS to user accounts, passwords, or personal data.".to_string();
    }else{
        system_prompt = format!("You are a specialized AI assistant. You stay strictly on these topics: {}. If a user asks about other topics, you MUST state that you do not have access and cannot help with those. NEVER pretend to have information outside these topics. You also DO NOT HAVE ACCESS to user accounts, passwords, or personal data.", topics);
    }
    

    let mut messages = Vec::new();
    
    // ALWAYS include the system prompt first
    messages.push(ChatMessage {
        role: "system".to_string(),
        content: system_prompt,
    });

    // Then add conversation history or current prompt
    if let Some(history) = &body.messages {
        // Add all messages from history (which should NOT include system prompt from client)
        messages.extend(history.iter().cloned());
    } else {
        // First message - just add the user's prompt
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: body.prompt.clone(),
        });
    }

    let payload = serde_json::json!({
        "model": llammacpp_model,
        "messages": messages,
        "stream": false
    });
    println!("[{}] Sending to llama.cpp (via /v1/chat/completions): {}", start_time.format("%H:%M:%S"), payload);

    let resp_result = client
        .post(format!("{}/v1/chat/completions", llammacpp_host))
        .json(&payload)
        .send()
        .await;

    let response_text = match resp_result {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => {
                // OpenAI format: choices[0].message.content
                json["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("No content field found in first choice")
                    .to_string()
            },
            Err(_) => "Invalid response from llama.cpp".to_string(),
        },
        Err(_) => "Failed to contact llama.cpp".to_string(),
    };

    let end_time = Local::now();
    println!(
        "[{}] Finished request ({} -> {})",
        end_time.format("%H:%M:%S"),
        start_time.format("%H:%M:%S"),
        end_time.format("%H:%M:%S")
    );

    HttpResponse::Ok().json(llamacppResponse { response: response_text })
}

async fn wait_for_llama(llammacpp_host: &str, client: &Client) {
    loop {
        match client.get(format!("{}/health", llammacpp_host)).send().await {
            Ok(resp) if resp.status().is_success() => break,
            _ => {
                println!("Waiting for llama.cpp to be ready...");
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let semaphore = Arc::new(Semaphore::new(1));
    let client = Client::new();
    let llammacpp_host = std::env::var("LLAMACPP_HOST").unwrap_or_else(|_| "http://llamacpp:11434".to_string());

    // Wait for llama.cpp before starting the server
    wait_for_llama(&llammacpp_host, &client).await;

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
