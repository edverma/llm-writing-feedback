#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri;
use tauri::Emitter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Listener;

#[tauri::command]
async fn process_file(file_path: String) -> Result<String, String> {
    // Load environment variables
    dotenv::dotenv().ok();
    // Retrieve API key from environment (currently unused in this stub)
    let _api_key =
        std::env::var("ANTHROPIC_API_KEY").map_err(|e| format!("API key error: {}", e))?;

    // Read the file contents asynchronously
    let content = tokio::fs::read_to_string(&file_path)
        .await
        .map_err(|err| format!("Error reading file: {}", err))?;

    // TODO: Integrate with the Anthropic API to process the file content and generate feedback
    // For now, simply return the file content as a placeholder
    Ok(content)
}

#[tauri::command]
async fn stop_feedback_monitor(app_handle: tauri::AppHandle) -> Result<(), String> {
    let _ = app_handle.emit("stop-monitoring", ());
    Ok(())
}

#[tauri::command]
async fn start_feedback_monitor(file_path: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    tokio::spawn(async move {
        dotenv::dotenv().ok();
        let api_key = match std::env::var("ANTHROPIC_API_KEY") {
            Ok(key) => key,
            Err(err) => {
                eprintln!("ANTHROPIC_API_KEY not set: {}", err);
                return;
            },
        };
        let api_url = "https://api.anthropic.com/v1/messages";
        let api_model = "claude-3-5-sonnet-20241022";
        let client = reqwest::Client::new();
        let mut messages = vec![];
        let mut cur_content = String::new();
        let mut version_num = 1;
        const MAX_NUM_DRAFTS: usize = 2;

        // Set up stop signal handling
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_signal_clone = stop_signal.clone();
        
        let _stop_handler = app_handle.listen("stop-monitoring", move |_| {
            stop_signal_clone.store(true, Ordering::SeqCst);
        });

        while !stop_signal.load(Ordering::SeqCst) {
            let new_content = match tokio::fs::read_to_string(&file_path).await {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file: {}", err);
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }
            };

            if new_content.trim() == cur_content.trim() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                continue;
            }
            cur_content = new_content.clone();

            messages.push(serde_json::json!({
                "role": "user",
                "content": cur_content,
            }));

            if messages.len() > MAX_NUM_DRAFTS {
                messages = messages.drain(messages.len() - MAX_NUM_DRAFTS..).collect();
            }
            
            let payload = serde_json::json!({
                "model": api_model,
                "max_tokens": 1024,
                "system": "You are a robot that lives inside of a simple markdown text editor. The user is unable to directly respond to your messages. The user is currently writing an article. You will be given each consecutive version of the article. The user is currently actively writing and editing the article. Upon reading the article, you should offer VERY BRIEF, but helpful, feedback on the latest version of the article. Only comment on the changed content between the latest version and the previous versions of the article.",
                "messages": messages,
            });

            // Emit loading state before making API call
            let _ = app_handle.emit("loading-state", true);

            let response_result = client
                .post(api_url)
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&payload)
                .send()
                .await;

            // Emit loading state complete after getting response
            let _ = app_handle.emit("loading-state", false);

            match response_result {
                Ok(resp) => {
                    let json_resp: serde_json::Value = match resp.json().await {
                        Ok(j) => j,
                        Err(err) => {
                            eprintln!("Error parsing json: {}", err);
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            continue;
                        }
                    };
                    if let Some(array) = json_resp["content"].as_array() {
                        if let Some(first_item) = array.first() {
                            if let Some(feedback_text) = first_item["text"].as_str() {
                                let payload = serde_json::json!({
                                    "version": version_num,
                                    "feedback": feedback_text
                                });
                                let _ = app_handle.emit("feedback-update", payload);
                            }
                        }
                    }
                    version_num += 1;
                },
                Err(err) => {
                    eprintln!("HTTP request error: {}", err);
                }
            }
        }
        
        // Emit stopped event when monitoring ends
        let _ = app_handle.emit("monitoring-stopped", ());
    });
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![process_file, start_feedback_monitor, stop_feedback_monitor])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
