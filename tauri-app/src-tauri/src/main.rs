#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::State;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};

#[derive(Default)]
struct MessageStore(Mutex<Vec<serde_json::Value>>);

#[derive(Serialize, Deserialize)]
struct AiResponse {
    content: String,
    version: usize,
}

#[tauri::command]
async fn analyze_content(
    content: String,
    message_store: State<'_, MessageStore>,
    api_key: State<'_, String>,
) -> Result<AiResponse, String> {
    let api_url = "https://api.anthropic.com/v1/messages";
    let api_model = "claude-3-5-sonnet-20241022";
    const MAX_NUM_DRAFTS: usize = 2;

    let messages = {
        let mut store = message_store.0.lock().unwrap();
        store.push(serde_json::json!({
            "role": "user",
            "content": content
        }));
        
        // Calculate the drain range before draining
        let store_len = store.len();
        if store_len > MAX_NUM_DRAFTS {
            store.drain(..store_len - MAX_NUM_DRAFTS);
        }
        
        store.clone()
    };
    
    let payload = serde_json::json!({
        "model": api_model,
        "max_tokens": 1024,
        "system": "You are a robot that lives inside of a simple markdown text editor. The user is unable to directly respond to your messages. The user is currently writing an article. You will be given the each consecutive version of the article. The user is currently actively writing and editing the article. Upon reading the article, you should offer VERY BRIEF, but helpful, feedback on the latest version of the article. Only comment on the changed content between the latest version and the previous versions of the article.",
        "messages": messages
    });

    let client = reqwest::Client::new();
    let response = client
        .post(api_url)
        .header("x-api-key", api_key.as_str())
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;

    let content = response["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|content| content["text"].as_str())
        .ok_or("Failed to get response content")?
        .to_string();

    Ok(AiResponse {
        content,
        version: messages.len(),
    })
}

fn main() {
    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| {
        eprintln!("Warning: ANTHROPIC_API_KEY not set. Using dummy value for development.");
        "dummy_key".to_string()
    });
    
    tauri::Builder::default()
        .manage(MessageStore::default())
        .manage(api_key)
        .invoke_handler(tauri::generate_handler![analyze_content])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}