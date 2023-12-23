use clap::{Parser, Subcommand};
use rustyline::{error::ReadlineError, history::FileHistory};
use serde_json::json;

struct ChatGPT {
    client: reqwest::blocking::Client,
    token: String,
    editor: rustyline::Editor<(), FileHistory>,
}

#[derive(Parser)]
#[command(author, version, about)]
struct CmdLineArgs {
    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(Subcommand)]
enum Action {
    Chat,
    Cmd,
}

fn add_assistant_message(messages: &mut Vec<serde_json::Value>, content: &str) {
    messages.push(json!({"role": "assistant", "content": content}));
}

fn add_user_message(messages: &mut Vec<serde_json::Value>, content: &str) {
    messages.push(json!({"role": "user", "content": content}));
}

fn add_system_message(messages: &mut Vec<serde_json::Value>, content: &str) {
    messages.push(json!({"role": "system", "content": content}));
}

fn get_response_text(server_response: &serde_json::Value) -> &str {
    let reply = &server_response["choices"][0]["message"];
    reply["content"]
        .as_str()
        .unwrap_or_else(|| panic!("content is not a string, response: {}", server_response))
        .trim()
}

impl ChatGPT {
    fn prompt(&self, messages: &Vec<serde_json::Value>, temperature: f32) -> serde_json::Value {
        self.client
            .execute(
                self.client
                    .post("https://api.openai.com/v1/chat/completions")
                    .json(&json!({
                        "temperature": temperature,
                        "model": "gpt-3.5-turbo",
                        "messages": messages,
                    }))
                    .header("Authorization", &self.token)
                    .build()
                    .unwrap(),
            )
            .unwrap()
            .json()
            .unwrap()
    }

    fn input(&mut self, prompt: &str, initial: &str) -> Result<String, ()> {
        match self.editor.readline_with_initial(prompt, (initial, "")) {
            Err(err) => Err(match err {
                ReadlineError::Eof | ReadlineError::Interrupted => (),
                _ => panic!("{}", err),
            }),
            Ok(line) => {
                self.editor.add_history_entry(&line).unwrap();
                Ok(line)
            }
        }
    }

    fn chat(&mut self) {
        let mut used_tokens_amount = 0;
        println!("(i) Enter an empty line to stop");
        let mut messages = Vec::new();
        loop {
            let Ok(input) = self.input(">>> ", "") else {
                break;
            };
            if input.is_empty() {
                println!("Used tokens amount: {}", used_tokens_amount);
                break;
            } else {
                add_user_message(&mut messages, &input);
                let response = self.prompt(&messages, 0.5);
                used_tokens_amount += response["usage"]["total_tokens"].as_u64().unwrap();
                let content = get_response_text(&response);
                println!("{}", content);
                add_assistant_message(&mut messages, content);
            }
        }
    }

    fn cmd(&mut self) {
        let Ok(input) = self.input("Input the description of a command: ", "") else {
            return;
        };
        let os_info = os_info::get();
        let mut messages = Vec::new();
        add_system_message(&mut messages, "Reply only with the shell command, do not explain anything - any response from you is acceptable. Do not format your answer.");
        add_user_message(
            &mut messages,
            &format!("My OS is {os_info}. Write a command to {input}."),
        );
        let response = self.prompt(&messages, 0.0);
        let response = get_response_text(&response);
        println!("You can edit the command before applying it:");
        let Ok(input) = self.input("> ", response) else {
            return;
        };
        subprocess::Exec::shell(input).join().unwrap();
    }
}

fn main() {
    let args = CmdLineArgs::parse();
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
    let client = reqwest::blocking::Client::builder()
        .timeout(None)
        .build()
        .unwrap();
    let mut chatgpt = ChatGPT {
        client,
        token,
        editor: rustyline::DefaultEditor::new().unwrap(),
    };
    match args.action {
        None => chatgpt.chat(),
        Some(action) => match action {
            Action::Chat => chatgpt.chat(),
            Action::Cmd => chatgpt.cmd(),
        },
    }
}
