use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt as _};
use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};
use std::io::stdin;
use std::io::Write;
use colored::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    username: String,
    text: String,
    timestamp: u64,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();

    println!("Enter username");
    let mut name = String::new();
    stdin().read_line(&mut name)
        .context("Failed to read username")?;

    let name = name.trim().to_string();
    let namet = name.clone();


    let (mut ws_stream, response) = connect_async(args[1].as_str())
        .await.context("Can't connect to server")?;

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
          
    let mut read_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(text.as_str()) {
                        let time_str = chrono::DateTime::from_timestamp(chat_msg.timestamp as i64, 0)
                            .map(|dt| dt.format("%H:%M:%S").to_string())
                            .unwrap_or_else(|| "??:??:??".to_string());
                    
                        if namet ==  chat_msg.username {
                            println!("\n[{}] {}: {}", 
                            time_str,
                            chat_msg.username.green(),
                            chat_msg.text);
                        } else {
                            println!("\n[{}] {}: {}", 
                            time_str,
                            chat_msg.username,
                            chat_msg.text);
                        };
                    
                        print!("> ");
                        let _ = std::io::stdout().flush();
                    }
                    else {println!("WTF");}
        
                }
                Ok(Message::Close(_)) => {
                    println!("Close connection");
                    break;
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        } 
    });

    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    let mut send_task = tokio::spawn(async move {
        while let Ok(Some(line)) = lines.next_line().await {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line == "/exit" {
                println!("Exiting...");
                let _ = ws_sender.send(Message::Close(None)).await;
                break;
            }

            let message = ChatMessage {
                username: name.clone(),
                text: line.to_string(),
                timestamp: chrono::Utc::now().timestamp() as u64,
            };

            if let Ok(json) = serde_json::to_string(&message) {
                let _ = ws_sender.send(Message::Text(json.into())).await
                    .context("Failed to send message");

                print!("> ");
                let _ = std::io::stdout().flush();
            }
        }
    });

    tokio::select! {
        _ = (&mut read_task) => send_task.abort(),
        _ = (&mut send_task) => read_task.abort(),
    };

    Ok(())
}