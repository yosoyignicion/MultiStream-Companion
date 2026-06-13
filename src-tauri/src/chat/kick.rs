// src-tauri/src/chat/kick.rs
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tauri::Emitter;

#[derive(Deserialize)]
struct KickChannelInfo {
    chatroom: KickChatroom,
}

#[derive(Deserialize)]
struct KickChatroom {
    id: u64,
}

#[derive(Serialize, Clone)]
pub struct UnifiedChatMessage {
    pub id: String,
    pub platform: String,
    pub user: String,
    pub text: String,
    pub color: String,
}

pub async fn fetch_kick_chatroom_id(username: &str) -> Result<u64, String> {
    let client = reqwest::Client::new();
    let url = format!("https://kick.com/api/v1/channels/{}", username);

    let response = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Error de red al consultar Kick: {}", e))?;

    let channel_info: KickChannelInfo = response.json()
        .await
        .map_err(|e| format!("Error al decodificar respuesta de Kick: {}", e))?;

    Ok(channel_info.chatroom.id)
}

pub async fn start_kick_listener(
    username: String,
    manual_chatroom_id: String,
    app_handle: tauri::AppHandle
) {
    // 1. Resolver ID
    let chatroom_id = if !manual_chatroom_id.is_empty() {
        manual_chatroom_id.parse::<u64>().unwrap_or_else(|_| {
            eprintln!("[KICK DEBUG] ID manual invalido, reintentando automatico.");
            0
        })
    } else {
        0
    };

    let chatroom_id = if chatroom_id != 0 {
        chatroom_id
    } else {
        match fetch_kick_chatroom_id(&username).await {
            Ok(id) => id,
            Err(e) => {
                eprintln!("[KICK DEBUG] Error al resolver ID automatico: {}. Bucle cancelado.", e);
                return;
            }
        }
    };

    println!("[KICK DEBUG] Conectando chat de Kick. ID RESUELTO: {}", chatroom_id);

    let url = "wss://ws-us2.pusher.com/app/32cbd69e4b950bf97679?protocol=7&client=js&version=8.4.0-rc2&flash=false";

    // 2. Conexion WebSocket
    let (ws_stream, _) = match connect_async(url).await {
        Ok(val) => val,
        Err(e) => {
            eprintln!("[KICK DEBUG ERROR] Error de handshake de red: {}", e);
            return;
        }
    };

    println!("[KICK DEBUG] Conexion WebSocket establecida con Pusher.");
    let (mut write, mut read) = ws_stream.split();

    // 3. Payload de suscripcion simplificado sin campo auth
    let subscribe_payload = json!({
        "event": "pusher:subscribe",
        "data": {
            "channel": format!("chatrooms.{}.v2", chatroom_id)
        }
    });

    if let Err(e) = write.send(Message::Text(subscribe_payload.to_string())).await {
        eprintln!("[KICK DEBUG ERROR] Fallo al enviar suscripcion: {}", e);
        return;
    }

    println!("[KICK DEBUG] Mensaje de suscripcion enviado al canal v2. Iniciando bucle de lectura...");

    // 4. Bucle de lectura super robusto que atrapa errores de socket
    while let Some(msg_result) = read.next().await {
        match msg_result {
            Ok(message) => {
                if let Message::Text(text) = message {
                    // Imprimimos TODO lo que llega de Pusher
                    println!("[DEBUG KICK RAW] {}", text);

                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        // Procesar el evento de mensaje
                        if parsed["event"] == "App\\Events\\ChatMessageEvent" {
                            if let Some(data_str) = parsed["data"].as_str() {
                                if let Ok(data) = serde_json::from_str::<serde_json::Value>(data_str) {
                                    let msg = UnifiedChatMessage {
                                        id: data["id"].as_str().unwrap_or_default().to_string(),
                                        platform: "kick".to_string(),
                                        user: data["sender"]["username"].as_str().unwrap_or_default().to_string(),
                                        text: data["content"].as_str().unwrap_or_default().to_string(),
                                        color: data["sender"]["identity"]["color"].as_str().unwrap_or("#53FC18").to_string(),
                                    };
                                    let _ = app_handle.emit("chat-message", msg);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[KICK DEBUG ERROR] Conexion interrumpida o cerrada por error: {}", e);
                break;
            }
        }
    }

    println!("[KICK DEBUG] El bucle de lectura de Kick ha terminado.");
}
