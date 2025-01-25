#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    dotenv::dotenv().ok();
    
    let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");

    let api_url = "https://api.anthropic.com/v1/messages";
    let api_model = "claude-3-5-sonnet-20241022";


    let file_path = &args[1];
    let mut new_content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            std::process::exit(1);
        }
    };
    let mut cur_content = String::new();
    let mut messages = vec![];
    let client = reqwest::Client::new();
    let mut version_num = 1;
    const MAX_NUM_DRAFTS: usize = 2;

    loop {
        while new_content.trim() == cur_content.trim() {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            new_content = match std::fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", file_path, e);
                    std::process::exit(1);
                }
            };
        }
        cur_content = new_content.clone();
        
        messages.push(serde_json::json!({
            "role": "user",
            "content": cur_content
        }));
        
        // Keep only the last MAX_NUM_DRAFTS messages
        if messages.len() > MAX_NUM_DRAFTS {
            messages = messages.drain(messages.len() - MAX_NUM_DRAFTS..).collect();
        }
        
        let payload = serde_json::json!({
            "model": api_model,
            "max_tokens": 1024,
            "system": "You are a robot that lives inside of a simple markdown text editor. The user is unable to directly respond to your messages. The user is currently writing an article. You will be given the
            each consecutive version of the article. The user is currently actively writing and editing the article.
            Upon reading the article, you should offer VERY BRIEF, but helpful, feedback on the latest version of the article. Only comment on the changed content between the latest version and the previous versions of the article.",
            "messages": messages
        });

        // Create and start the progress bar
        let spinner = indicatif::ProgressBar::new_spinner();
        spinner.set_message("Waiting for AI response...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(120));

        // Make the API call
        let response = client
            .post(api_url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // Stop the spinner
        spinner.finish_and_clear();

        // Print only the content from the response
        if let Some(content) = response["content"].as_array().and_then(|arr| arr.first()) {
            if let Some(text) = content["text"].as_str() {
                // Clear the terminal screen
                print!("\x1B[2J\x1B[1;1H");  // ANSI escape codes to clear screen and move cursor to top
                println!("Writing Feedback (Version {}):\n\n{}\n\n", version_num, text);
            }
        }
        version_num += 1;
    }
}
