use std::path::Path;
use std::sync::OnceLock;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    if !std::path::Path::new("./settings.json").exists() {
        match std::fs::write(
            "./settings.json",
            serde_json::json!({
                "token": "TOKEN_HERE",
                "files_dir": "/opt/www/files/",
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

        static TOKEN: OnceLock<String> = OnceLock::new();
        let _ = TOKEN.get_or_init(|| {
            settings["token"]
                .as_str()
                .expect("Token not found in settings.json")
                .to_string()
        });
        static FILES_DIR: OnceLock<String> = OnceLock::new();
        let _ = FILES_DIR.get_or_init(|| {
            settings["files_dir"]
                .as_str()
                .expect("Files directory not found in settings.json")
                .to_string()
        });
        static URL: OnceLock<String> = OnceLock::new();
        let _ = URL.get_or_init(|| {
            settings["url"]
                .as_str()
                .expect("URL not found in settings.json")
                .to_string()
        });

        let bot = Bot::new(TOKEN.get().unwrap());
        std::fs::create_dir_all(FILES_DIR.get().unwrap()).expect("Error creating files directory");

        teloxide::repl(bot, |bot: Bot, msg: Message| async move {
            if msg.document().is_none() {
                bot.send_message(msg.chat.id, "Processing text...")
                    .send()
                    .await
                    .unwrap();
                if msg.text().is_none() {
                    bot.send_message(msg.chat.id, "Please send a file or text!")
                        .send()
                        .await
                        .unwrap();
                } else {
                    let file_contents = msg.text().unwrap();
                    let hash = format!("{:x}", md5::compute(file_contents.as_bytes())).to_string();
                    let file_path = Path::new(FILES_DIR.get().unwrap()).join(hash.clone());
                    std::fs::write(&file_path, file_contents).unwrap();
                    bot.send_message(
                        msg.chat.id,
                        format!("File saved, checkout {}{}!", URL.get().unwrap(), hash),
                    )
                    .send()
                    .await
                    .unwrap();
                }
            } else {
                bot.send_message(msg.chat.id, "Processing file...")
                    .send()
                    .await
                    .unwrap();
                let file = msg.document().unwrap();

                let file_metadata = reqwest::get(format!(
                    "https://api.telegram.org/bot{}/getFile?file_id={}",
                    TOKEN.get().unwrap(),
                    file.file.id
                ))
                .await
                .unwrap();
                let file_metadata: serde_json::Value =
                    serde_json::from_str(&file_metadata.text().await.unwrap().to_string()).unwrap();
                let file_path = file_metadata["result"]["file_path"]
                    .as_str()
                    .expect("Error getting file path");
                let extention = match Path::new(file_path).extension() {
                    Some(ext) => ext.to_str().unwrap().to_string(),
                    None => "".to_string(),
                };

                let one_more_json = reqwest::get(format!(
                    "https://api.telegram.org/file/bot{}/{}",
                    TOKEN.get().unwrap(),
                    file_path
                ))
                .await
                .unwrap();
                let file_url = one_more_json.url().to_string();
                let file_content = reqwest::get(file_url).await.unwrap().bytes().await.unwrap();
                let hash = format!("{:x}", md5::compute(&file_content)).to_string();
                let file_path = Path::new(FILES_DIR.get().unwrap())
                    .join(hash.clone().to_string() + "." + &extention);
                std::fs::write(&file_path, file_content).unwrap();
                let extention = if extention.is_empty() {
                    "".to_string()
                } else {
                    ".".to_string() + &extention
                };
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "File saved, checkout {}{}{}!",
                        URL.get().unwrap(),
                        hash,
                        extention
                    ),
                )
                .send()
                .await
                .unwrap();
            }

            Ok(())
        })
        .await;
    }
}
