use std::io::Write;

use serde_json::json;

fn error_panic(response: &serde_json::Value) -> ! {
    panic!("There was an error requesting an API: {:?}", response);
}

fn main() {
    let token_path = home::home_dir()
        .expect("Home directory path not found")
        .join(".config/chatgpt_cli/openai_token.txt");
    let token = std::fs::read_to_string(&token_path).unwrap_or_else(|err| {
        panic!(
            "Couldn't read the token file at {:?}: {:?}",
            token_path, err
        )
    });
    let token = format!("Bearer {}", token.trim());
    let client = reqwest::blocking::Client::new();
    let mut used_tokens_amount = 0;
    println!("(i) Enter an empty line to stop");
    let mut messages = Vec::new();
    loop {
        let mut input = String::new();
        print!(">>> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.is_empty() {
            println!("Used tokens amount: {}", used_tokens_amount);
            break;
        } else {
            messages.push(json!({"role": "user", "content": input}));
            let response: serde_json::Value = client
                .execute(
                    client
                        .post("https://api.openai.com/v1/chat/completions")
                        .json(&json!({
                            "temperature": 0.5,
                            "model": "gpt-3.5-turbo",
                            "messages": messages,
                        }))
                        .header("Authorization", &token)
                        .build()
                        .unwrap(),
                )
                .unwrap()
                .json()
                .unwrap();
            used_tokens_amount += response
                .get("usage")
                .and_then(|usage| usage.get("total_tokens"))
                .map(|total_tokens| total_tokens.as_u64().unwrap())
                .unwrap_or_else(|| error_panic(&response));
            let reply = response
                .get("choices")
                .and_then(|choices| choices.get(0))
                .and_then(|choice| choice.get("message"))
                .unwrap_or_else(|| error_panic(&response));
            let content = reply["content"].as_str().unwrap().trim();
            println!("{}", content);
            messages.push(json!({"role": "assistant", "content": content}));
        }
    }
}
