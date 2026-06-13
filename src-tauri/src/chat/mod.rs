pub mod kick;
pub mod twitch;

use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use crate::config::{get_secret_internal, load_config_internal};

#[derive(Default)]
pub struct ChatState {
    pub active_listeners: Arc<Mutex<Vec<String>>>,
}

#[tauri::command]
pub async fn start_chat_ingestion(
    platform: String,
    username: String,
    app_handle: tauri::AppHandle,
    state: State<'_, ChatState>,
) -> Result<String, String> {
    let mut listeners = state.active_listeners.lock().await;

    if listeners.contains(&platform) {
        return Ok(format!("La escucha para {} ya esta activa", platform));
    }

    let config = load_config_internal(&app_handle)?;

    match platform.as_str() {
        "kick" => {
            let app = app_handle.clone();
            let manual_id = config.kick_chatroom_id.clone();
            tokio::spawn(async move {
                kick::start_kick_listener(username, manual_id, app).await;
            });
            listeners.push(platform.clone());
        }
        "twitch" => {
            let oauth_token = get_secret_internal("twitch_oauth_token")
                .map_err(|e| format!("Falta el token de Twitch en el llavero: {}", e))?;

            if oauth_token.is_empty() {
                return Err(
                    "No se encontro un token de Twitch en el llavero".to_string(),
                );
            }

            let app = app_handle.clone();
            tokio::spawn(async move {
                twitch::start_twitch_listener(username, oauth_token, app).await;
            });
            listeners.push(platform.clone());
        }
        _ => return Err("Plataforma de chat no soportada".to_string()),
    }

    Ok(format!("Iniciada la escucha de chat para {}", platform))
}
