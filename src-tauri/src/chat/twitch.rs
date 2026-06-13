use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::chat::kick::UnifiedChatMessage;

pub async fn start_twitch_listener(
    username: String,
    oauth_token: String,
    app_handle: tauri::AppHandle,
) {
    let url = "wss://irc-ws.chat.twitch.tv:443";

    let (ws_stream, _) = match connect_async(url).await {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error al conectar con WebSocket de Twitch: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    let _ = write
        .send(Message::Text(format!("PASS oauth:{}", oauth_token)))
        .await;
    let _ = write
        .send(Message::Text(format!("NICK {}", username)))
        .await;
    let _ = write
        .send(Message::Text(format!("JOIN #{}", username.to_lowercase())))
        .await;

    use tauri::Emitter;
    eprintln!("Escuchando chat de Twitch para: {}", username);

    while let Some(Ok(message)) = read.next().await {
        if let Message::Text(text) = message {
            if text.starts_with("PING") {
                let _ = write
                    .send(Message::Text("PONG :tmi.twitch.tv".to_string()))
                    .await;
                continue;
            }

            if text.contains("PRIVMSG") {
                if let Some((user, msg)) = parse_irc_message(&text) {
                    let message = UnifiedChatMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        platform: "twitch".to_string(),
                        user,
                        text: msg,
                        color: "#9146FF".to_string(),
                    };
                    let _ = app_handle.emit("chat-message", message);
                }
            }
        }
    }
}

fn parse_irc_message(raw: &str) -> Option<(String, String)> {
    if !raw.starts_with(':') {
        return None;
    }

    let parts: Vec<&str> = raw.splitn(2, " PRIVMSG ").collect();
    if parts.len() < 2 {
        return None;
    }

    let user_part = parts[0];
    let user = user_part.split('!').next()?.trim_start_matches(':');

    let msg_part = parts[1];
    let msg_split: Vec<&str> = msg_part.splitn(2, " :").collect();
    if msg_split.len() < 2 {
        return None;
    }
    let msg = msg_split[1].trim_end();

    Some((user.to_string(), msg.to_string()))
}
