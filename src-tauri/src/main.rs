#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![process_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
