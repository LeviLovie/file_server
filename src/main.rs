use std::path::Path;
use std::sync::OnceLock;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    // Check if file ./settings.json exists
    if !std::path::Path::new("./settings.json").exists() {
        match std::fs::write(
            "./settings.json",
            serde_json::json!({
                "token": "TOKEN_HERE",
                "files_dir": "/opt/files/",
                "url": "https://files.lovie.dev/"
            })
            .to_string(),
        ) {
            Ok(_) => {
                println!("Please fill in the settings.json file");
                std::process::exit(1);
            }
            Err(e) => {
                println!("Error writing settings.json: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        let settings: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string("./settings.json").expect("Error reading settings.json"),
        )
        .expect("Error parsing settings.json");

        let token = settings["token"].as_str().unwrap();
        static FILES_DIR: OnceLock<String> = OnceLock::new();
        let _ = FILES_DIR.get_or_init(|| {
            settings["files_dir"]
                .to_string()
                .trim_start_matches("\"")
                .trim_end_matches("\"")
                .to_string()
        });
        static URL: OnceLock<String> = OnceLock::new();
        let _ = URL.get_or_init(|| {
            settings["url"]
                .to_string()
                .trim_start_matches("\"")
                .trim_end_matches("\"")
                .to_string()
        });

        let bot = Bot::new(token);
        std::fs::create_dir_all(FILES_DIR.get().unwrap()).expect("Error creating files directory");

        teloxide::repl(bot, |bot: Bot, msg: Message| async move {
            match msg.text() {
                None => {
                    bot.send_message(msg.chat.id, "Please send a text message")
                        .send()
                        .await
                        .unwrap();
                }
                Some(text) => {
                    let file_name = format!("{:x}", md5::compute(text.as_bytes())).to_string();
                    let file_path = Path::new(FILES_DIR.get().unwrap()).join(file_name.clone());
                    std::fs::write(&file_path, text).unwrap();
                    bot.send_message(
                        msg.chat.id,
                        format!("File saved, checkout {}{}!", URL.get().unwrap(), file_name),
                    )
                    .send()
                    .await
                    .unwrap();
                }
            }

            Ok(())
        })
        .await;
    }
}
