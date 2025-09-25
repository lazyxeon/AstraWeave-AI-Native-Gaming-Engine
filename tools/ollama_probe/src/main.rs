use anyhow::Result;
use reqwest::blocking::Client;
use serde_json::Value;

fn main() -> Result<()> {
    let client = Client::builder().timeout(std::time::Duration::from_secs(15)).build()?;
    let url = "http://127.0.0.1:11434/api/chat";
    let body = serde_json::json!({
        "model": "phi3:medium",
        "messages": [{"role": "user", "content": "Please reply with only JSON: {\"plan_id\":\"test-plan\",\"steps\":[]}"}],
        "stream": false
    });

    println!("POSTING to {}", url);
    let resp = client.post(url).json(&body).send()?;
    println!("Status: {}", resp.status());
    let text = resp.text()?;
    println!("Raw response:\n{}", text);

    if let Ok(v) = serde_json::from_str::<Value>(&text) {
        println!("Parsed JSON: {}", serde_json::to_string_pretty(&v)?);
    } else {
        println!("Response not JSON");
    }

    Ok(())
}
